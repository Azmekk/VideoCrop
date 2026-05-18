using System.Diagnostics;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;
using VideoCrop.Core.IO;

namespace VideoCrop.Core.Processes;

public sealed class MpvHostOptions
{
    public string WindowTitle { get; init; } = "VideoCrop - Preview";
    public string? InitialFile { get; init; }
}

public sealed class MpvHost(IToolLocator locator, ILogger<MpvHost>? logger = null) : IAsyncDisposable
{
    private readonly ILogger<MpvHost> _logger = logger ?? NullLogger<MpvHost>.Instance;
    private Process? _process;
    private MpvIpcClient? _ipc;
    private string _pipeName = "";
    private bool _disposed;

    public MpvIpcClient Ipc => _ipc ?? throw new InvalidOperationException("MpvHost not started.");
    public bool IsRunning => _process is { HasExited: false };

    /// <summary>Raised when the mpv process exits for any reason.</summary>
    public event EventHandler? Exited;

    public async Task StartAsync(MpvHostOptions options, CancellationToken cancellationToken)
    {
        if (_process is not null) throw new InvalidOperationException("Already started.");
        if (!locator.TryResolve(ExternalTool.Mpv, out var mpv))
            throw new InvalidOperationException("mpv not found.");

        _pipeName = $@"\\.\pipe\videocrop-mpv-{Guid.NewGuid():N}";

        var psi = new ProcessStartInfo
        {
            FileName = mpv,
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardError = true,
        };
        psi.ArgumentList.Add($"--input-ipc-server={_pipeName}");
        psi.ArgumentList.Add("--idle=yes");
        psi.ArgumentList.Add("--force-window=yes");
        psi.ArgumentList.Add("--keep-open=yes");
        psi.ArgumentList.Add("--pause");
        psi.ArgumentList.Add("--no-terminal");
        psi.ArgumentList.Add($"--title={options.WindowTitle}");
        // Standalone window: let mpv keep its on-screen controller and default
        // key bindings so it is usable on its own.
        if (!string.IsNullOrEmpty(options.InitialFile))
            psi.ArgumentList.Add(options.InitialFile);

        _logger.LogInformation("Spawning mpv: {Exe} {Args}", mpv, string.Join(' ', psi.ArgumentList));

        _process = new Process { StartInfo = psi, EnableRaisingEvents = true };
        _process.ErrorDataReceived += (_, e) =>
        {
            if (e.Data is not null) _logger.LogInformation("mpv: {Line}", e.Data);
        };
        _process.Exited += (_, _) =>
        {
            _logger.LogInformation("mpv process exited (code {Code})", _process?.ExitCode);
            Exited?.Invoke(this, EventArgs.Empty);
        };
        if (!_process.Start()) throw new InvalidOperationException("Failed to start mpv.");
        _logger.LogInformation("mpv spawned, pid={Pid}, pipe={Pipe}", _process.Id, _pipeName);
        _process.BeginErrorReadLine();

        _ipc = new MpvIpcClient(_pipeName, NullLogger<MpvIpcClient>.Instance);
        await _ipc.ConnectAsync(TimeSpan.FromSeconds(10), cancellationToken).ConfigureAwait(false);
        _logger.LogInformation("mpv IPC pipe connected");
    }

    public async Task LoadFileAsync(string path, CancellationToken ct = default)
    {
        await Ipc.SendCommandAsync(["loadfile", path, "replace"], ct).ConfigureAwait(false);
    }

    public async Task ScreenshotToFileAsync(string outputPath, CancellationToken ct = default)
    {
        await Ipc.SendCommandAsync(["screenshot-to-file", outputPath, "video"], ct).ConfigureAwait(false);
    }

    public async ValueTask DisposeAsync()
    {
        if (_disposed) return;
        _disposed = true;

        if (_ipc is not null)
        {
            try { await _ipc.SendCommandAsync(["quit"], CancellationToken.None).ConfigureAwait(false); }
            catch { /* mpv may already be dying */ }
            await _ipc.DisposeAsync().ConfigureAwait(false);
        }

        if (_process is not null)
        {
            try
            {
                if (!_process.HasExited && !_process.WaitForExit(1000))
                    _process.Kill(entireProcessTree: true);
            }
            catch { }
            _process.Dispose();
        }
    }
}
