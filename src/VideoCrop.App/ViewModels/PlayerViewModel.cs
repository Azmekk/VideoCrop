using System;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.Extensions.Logging;
using VideoCrop.Core.IO;
using VideoCrop.Core.Processes;

namespace VideoCrop.App.ViewModels;

public sealed class PlayerViewModel(IToolLocator locator, ILoggerFactory loggerFactory)
    : ObservableObject, IAsyncDisposable
{
    private readonly ILogger<PlayerViewModel> _logger = loggerFactory.CreateLogger<PlayerViewModel>();
    private readonly SynchronizationContext? _uiContext = SynchronizationContext.Current;

    private MpvHost? _host;
    private bool _isReady;
    private bool _isPaused = true;
    private double _positionSeconds;
    private double _durationSeconds;
    private string? _currentFile;
    private bool _suppressSeekFeedback;
    private bool _hasMpv = locator.TryResolve(ExternalTool.Mpv, out _);
    private TimeSpan? _cutStart;
    private TimeSpan? _cutEnd;
    private nint _videoHwnd;
    private double _volume = 100;
    private bool _suppressVolumeFeedback;

    public bool HasMpv { get => _hasMpv; private set => SetProperty(ref _hasMpv, value); }

    /// <summary>
    /// Native HWND of the child window mpv should render into. Must be set
    /// before <see cref="EnsureStartedAsync"/> is called; otherwise mpv runs
    /// in standalone-window mode.
    /// </summary>
    public nint VideoHwnd
    {
        get => _videoHwnd;
        set => _videoHwnd = value;
    }

    public bool IsReady { get => _isReady; private set => SetProperty(ref _isReady, value); }
    public bool IsPaused { get => _isPaused; private set => SetProperty(ref _isPaused, value); }
    public double PositionSeconds { get => _positionSeconds; private set => SetProperty(ref _positionSeconds, value); }
    public double DurationSeconds { get => _durationSeconds; private set => SetProperty(ref _durationSeconds, value); }
    public string? CurrentFile { get => _currentFile; private set => SetProperty(ref _currentFile, value); }
    /// <summary>0–100. Reflects mpv's <c>volume</c> property.</summary>
    public double Volume { get => _volume; private set => SetProperty(ref _volume, value); }

    public async Task<bool> EnsureStartedAsync(CancellationToken ct = default)
    {
        if (_host is { IsRunning: true }) return true;

        if (!locator.TryResolve(ExternalTool.Mpv, out var mpvPath))
        {
            HasMpv = false;
            _logger.LogWarning("mpv not found via ToolLocator");
            return false;
        }
        HasMpv = true;
        _logger.LogInformation("Starting mpv from {Path}", mpvPath);

        if (_host is not null)
        {
            await _host.DisposeAsync();
            _host = null;
        }

        _host = new MpvHost(locator, loggerFactory.CreateLogger<MpvHost>());
        _host.Exited += OnHostExited;
        try
        {
            await _host.StartAsync(new MpvHostOptions { ParentHwnd = _videoHwnd }, ct);
            _host.Ipc.PropertyChanged += OnPropertyChange;
            await _host.Ipc.ObservePropertyAsync("time-pos", ct);
            await _host.Ipc.ObservePropertyAsync("duration", ct);
            await _host.Ipc.ObservePropertyAsync("pause", ct);
            await _host.Ipc.ObservePropertyAsync("volume", ct);
            // Restore the last user-set volume on the fresh mpv instance.
            try { await _host.Ipc.SetPropertyAsync("volume", _volume); } catch { }
            IsReady = true;
            _logger.LogInformation("mpv ready (IPC connected)");
            return true;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to start mpv");
            await _host.DisposeAsync();
            _host = null;
            IsReady = false;
            return false;
        }
    }

    public async Task LoadFileAsync(string path, CancellationToken ct = default)
    {
        if (!await EnsureStartedAsync(ct))
        {
            _logger.LogWarning("Cannot load file because mpv could not be started: {Path}", path);
            return;
        }
        try
        {
            _logger.LogInformation("Loading {Path} into mpv", path);
            await _host!.LoadFileAsync(path, ct);
            CurrentFile = path;
            IsPaused = true;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to load file in mpv: {Path}", path);
        }
    }

    /// <summary>
    /// Re-spawn mpv and re-load the current file. Called when the user closes
    /// the standalone mpv window and wants to bring it back without re-picking
    /// the source.
    /// </summary>
    public async Task ReopenAsync()
    {
        if (_currentFile is null) return;
        if (!await EnsureStartedAsync()) return;
        try
        {
            await _host!.LoadFileAsync(_currentFile);
            IsPaused = true;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Reopen failed");
        }
    }

    public async Task TogglePauseAsync()
    {
        if (!await EnsureReadyForCommandsAsync()) return;
        try
        {
            await _host!.Ipc.SetPropertyAsync("pause", !IsPaused);
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Toggle pause failed");
        }
    }

    public async Task SetVolumeAsync(double volume)
    {
        var v = Math.Clamp(volume, 0, 100);
        Volume = v;
        if (!await EnsureReadyForCommandsAsync()) return;
        try
        {
            _suppressVolumeFeedback = true;
            await _host!.Ipc.SetPropertyAsync("volume", v);
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "SetVolume failed");
        }
        finally
        {
            _suppressVolumeFeedback = false;
        }
    }

    public async Task SeekAsync(double seconds)
    {
        if (!await EnsureReadyForCommandsAsync()) return;
        try
        {
            _suppressSeekFeedback = true;
            await _host!.Ipc.SeekClampedAsync(seconds);
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Seek failed");
        }
        finally
        {
            _suppressSeekFeedback = false;
        }
    }

    /// <summary>
    /// Ensures mpv is running and the current file (if any) is loaded.
    /// Called from interactive commands so the app silently recovers from a
    /// dead mpv process.
    /// </summary>
    private async Task<bool> EnsureReadyForCommandsAsync()
    {
        if (_host is { IsRunning: true } && IsReady) return true;
        if (!await EnsureStartedAsync()) return false;
        if (_currentFile is not null)
        {
            try { await _host!.LoadFileAsync(_currentFile); }
            catch (Exception ex) { _logger.LogWarning(ex, "Reload after respawn failed"); return false; }
        }
        return true;
    }

    public async Task<bool> ScreenshotToFileAsync(string outputPath)
    {
        if (_host is null) return false;
        try
        {
            await _host.Ipc.SetPropertyAsync("pause", true);
            await _host.ScreenshotToFileAsync(outputPath);
            return true;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Screenshot failed");
            return false;
        }
    }

    public async Task SetVideoCropAsync(string? value)
    {
        if (_host is null) return;
        try
        {
            await _host.Ipc.SetPropertyAsync("video-crop", value ?? "");
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "video-crop failed");
        }
    }

    public async Task ApplyCutBoundsAsync(TimeSpan? start, TimeSpan? end)
    {
        _cutStart = start;
        _cutEnd = end;
        if (_host is null) return;
        try
        {
            // Track bounds inside the IPC client for clamped seeks, and make
            // sure any previously-set A-B loop is cleared. We pause at the
            // end-bound via the time-pos observer below rather than loop.
            _host.Ipc.SetCutBounds(start, end);
            await _host.Ipc.SetPropertyAsync("ab-loop-a", "no");
            await _host.Ipc.SetPropertyAsync("ab-loop-b", "no");
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "ApplyCutBounds failed");
        }
    }

    private void OnHostExited(object? sender, EventArgs e)
    {
        // mpv died (closed by user, crashed, decoder error, whatever). Clear
        // our state so the next interaction re-spawns it via EnsureStartedAsync
        // and re-loads the current file.
        if (_uiContext is null) ApplyHostExited();
        else _uiContext.Post(_ => ApplyHostExited(), null);
    }

    private void ApplyHostExited()
    {
        _logger.LogInformation("mpv exited; will respawn on next interaction");
        IsReady = false;
        IsPaused = true;
        // Don't dispose _host here — DisposeAsync inside EnsureStartedAsync
        // will handle that on the next start. Clearing the reference lets
        // EnsureStartedAsync's `IsRunning` check fail and trigger respawn.
    }

    private void OnPropertyChange(object? sender, MpvPropertyChange e)
    {
        Action apply = () =>
        {
            switch (e.Name)
            {
                case "time-pos" when e.Data is JsonElement t && t.ValueKind == JsonValueKind.Number:
                    var pos = t.GetDouble();
                    if (!_suppressSeekFeedback) PositionSeconds = pos;
                    // Pause when we reach (or pass) the cut end during playback.
                    if (!IsPaused && _cutEnd is { } endBound && pos >= endBound.TotalSeconds - 0.05)
                    {
                        _ = PauseAtEndAsync(endBound);
                    }
                    break;
                case "duration" when e.Data is JsonElement d && d.ValueKind == JsonValueKind.Number:
                    DurationSeconds = d.GetDouble();
                    break;
                case "pause" when e.Data is JsonElement p:
                    IsPaused = p.ValueKind == JsonValueKind.True;
                    break;
                case "volume" when e.Data is JsonElement v && v.ValueKind == JsonValueKind.Number:
                    if (!_suppressVolumeFeedback) Volume = v.GetDouble();
                    break;
            }
        };

        if (_uiContext is null) apply();
        else _uiContext.Post(_ => apply(), null);
    }

    private async Task PauseAtEndAsync(TimeSpan endBound)
    {
        if (_host is null) return;
        try
        {
            // Pause and snap the position exactly to the cut end so the user
            // doesn't see an overshot frame.
            await _host.Ipc.SetPropertyAsync("pause", true);
            await _host.Ipc.SeekClampedAsync(endBound.TotalSeconds);
        }
        catch (Exception ex)
        {
            _logger.LogDebug(ex, "PauseAtEnd failed");
        }
    }

    public async ValueTask DisposeAsync()
    {
        if (_host is not null)
        {
            await _host.DisposeAsync();
            _host = null;
        }
        IsReady = false;
    }
}
