using System.Collections.Concurrent;
using System.IO.Pipes;
using System.Text.Json;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;

namespace VideoCrop.Core.Processes;

/// <summary>
/// Async IPC client for mpv's JSON IPC protocol over a named pipe.
/// On Windows the pipe is <c>\\.\pipe\&lt;name&gt;</c>.
/// </summary>
public sealed class MpvIpcClient(string pipeName, ILogger<MpvIpcClient>? logger = null) : IAsyncDisposable
{
    private static readonly JsonSerializerOptions JsonOptions = new(JsonSerializerDefaults.Web);

    private readonly ILogger<MpvIpcClient> _logger = logger ?? NullLogger<MpvIpcClient>.Instance;
    private NamedPipeClientStream? _pipe;
    private StreamReader? _reader;
    private StreamWriter? _writer;
    private Task? _readLoop;
    private readonly CancellationTokenSource _cts = new();

    private long _requestCounter;
    private readonly ConcurrentDictionary<long, TaskCompletionSource<JsonElement>> _pending = new();
    private readonly ConcurrentDictionary<long, string> _observedProperties = new();
    private long _observeCounter;
    private TimeSpan? _cutStart;
    private TimeSpan? _cutEnd;
    private bool _disposed;

    public event EventHandler<MpvPropertyChange>? PropertyChanged;
    public event EventHandler<string>? Event;

    public bool IsConnected => _pipe?.IsConnected == true;

    public async Task ConnectAsync(TimeSpan timeout, CancellationToken cancellationToken)
    {
        var stripped = StripPipePrefix(pipeName);
        _pipe = new NamedPipeClientStream(".", stripped, PipeDirection.InOut, PipeOptions.Asynchronous);

        using var linked = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        linked.CancelAfter(timeout);
        await _pipe.ConnectAsync(linked.Token).ConfigureAwait(false);
        _pipe.ReadMode = PipeTransmissionMode.Byte;
        _reader = new StreamReader(_pipe, System.Text.Encoding.UTF8, detectEncodingFromByteOrderMarks: false, bufferSize: 4096, leaveOpen: true);
        _writer = new StreamWriter(_pipe, new System.Text.UTF8Encoding(false)) { AutoFlush = true, NewLine = "\n" };
        _readLoop = Task.Run(() => ReadLoopAsync(_cts.Token));
    }

    public Task<JsonElement> SendCommandAsync(params object[] commandArgs) =>
        SendCommandAsync(commandArgs, CancellationToken.None);

    public async Task<JsonElement> SendCommandAsync(IEnumerable<object> commandArgs, CancellationToken cancellationToken)
    {
        EnsureConnected();
        var requestId = Interlocked.Increment(ref _requestCounter);
        var tcs = new TaskCompletionSource<JsonElement>(TaskCreationOptions.RunContinuationsAsynchronously);
        _pending[requestId] = tcs;

        var payload = new
        {
            command = commandArgs.ToArray(),
            request_id = requestId,
        };
        var line = JsonSerializer.Serialize(payload, JsonOptions);

        try
        {
            await _writer!.WriteLineAsync(line.AsMemory(), cancellationToken).ConfigureAwait(false);
        }
        catch
        {
            _pending.TryRemove(requestId, out _);
            throw;
        }

        using var registration = cancellationToken.Register(() =>
        {
            if (_pending.TryRemove(requestId, out var pending)) pending.TrySetCanceled(cancellationToken);
        });

        return await tcs.Task.ConfigureAwait(false);
    }

    public Task SetPropertyAsync(string name, object value, CancellationToken ct = default) =>
        SendCommandAsync(new object[] { "set_property", name, value }, ct);

    public async Task<JsonElement> GetPropertyAsync(string name, CancellationToken ct = default) =>
        await SendCommandAsync(new object[] { "get_property", name }, ct).ConfigureAwait(false);

    public async Task<long> ObservePropertyAsync(string name, CancellationToken ct = default)
    {
        var id = Interlocked.Increment(ref _observeCounter);
        _observedProperties[id] = name;
        await SendCommandAsync(new object[] { "observe_property", id, name }, ct).ConfigureAwait(false);
        return id;
    }

    public Task UnobservePropertyAsync(long observerId, CancellationToken ct = default)
    {
        _observedProperties.TryRemove(observerId, out _);
        return SendCommandAsync(new object[] { "unobserve_property", observerId }, ct);
    }

    public void SetCutBounds(TimeSpan? start, TimeSpan? end)
    {
        _cutStart = start;
        _cutEnd = end;
    }

    public Task SeekClampedAsync(double seconds, CancellationToken ct = default)
    {
        var min = _cutStart?.TotalSeconds ?? double.NegativeInfinity;
        var max = _cutEnd?.TotalSeconds ?? double.PositiveInfinity;
        var clamped = Math.Clamp(seconds, min == double.NegativeInfinity ? seconds : min, max == double.PositiveInfinity ? seconds : max);
        return SendCommandAsync(new object[] { "seek", clamped, "absolute", "exact" }, ct);
    }

    private async Task ReadLoopAsync(CancellationToken ct)
    {
        try
        {
            while (!ct.IsCancellationRequested && _reader is not null)
            {
                var line = await _reader.ReadLineAsync(ct).ConfigureAwait(false);
                if (line is null) break;
                if (string.IsNullOrWhiteSpace(line)) continue;
                HandleLine(line);
            }
        }
        catch (OperationCanceledException)
        {
            // expected on disposal
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "mpv IPC read loop terminated");
        }
    }

    private void HandleLine(string line)
    {
        JsonElement root;
        try
        {
            using var doc = JsonDocument.Parse(line);
            root = doc.RootElement.Clone();
        }
        catch (JsonException ex)
        {
            _logger.LogWarning(ex, "Skipping malformed IPC line: {Line}", line);
            return;
        }

        if (root.TryGetProperty("event", out var evt))
        {
            var name = evt.GetString();
            if (name == "property-change" && root.TryGetProperty("name", out var n))
            {
                var propName = n.GetString() ?? "";
                JsonElement? data = root.TryGetProperty("data", out var d) ? d : null;
                PropertyChanged?.Invoke(this, new MpvPropertyChange(propName, data));
            }
            else if (name is not null)
            {
                Event?.Invoke(this, name);
            }
            return;
        }

        if (root.TryGetProperty("request_id", out var reqId) && reqId.TryGetInt64(out var id))
        {
            if (_pending.TryRemove(id, out var tcs))
            {
                if (root.TryGetProperty("error", out var err) && err.GetString() != "success")
                {
                    tcs.TrySetException(new InvalidOperationException($"mpv error: {err.GetString()}"));
                }
                else
                {
                    var data = root.TryGetProperty("data", out var dataEl) ? dataEl : default;
                    tcs.TrySetResult(data);
                }
            }
        }
    }

    private void EnsureConnected()
    {
        if (_disposed) throw new ObjectDisposedException(nameof(MpvIpcClient));
        if (_pipe is null || !_pipe.IsConnected)
            throw new InvalidOperationException("mpv IPC not connected.");
    }

    private static string StripPipePrefix(string raw)
    {
        const string prefix = @"\\.\pipe\";
        return raw.StartsWith(prefix, StringComparison.Ordinal) ? raw[prefix.Length..] : raw;
    }

    public async ValueTask DisposeAsync()
    {
        if (_disposed) return;
        _disposed = true;
        try { await _cts.CancelAsync().ConfigureAwait(false); } catch { }
        try { if (_readLoop is not null) await _readLoop.ConfigureAwait(false); } catch { }
        _reader?.Dispose();
        if(_writer is not null) await _writer.DisposeAsync();
        if (_pipe is not null) await _pipe.DisposeAsync();
        _cts.Dispose();
    }
}

public sealed record MpvPropertyChange(string Name, JsonElement? Data);
