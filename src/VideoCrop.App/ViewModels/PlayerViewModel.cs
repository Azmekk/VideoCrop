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

    public bool HasMpv { get => _hasMpv; private set => SetProperty(ref _hasMpv, value); }
    public bool IsReady { get => _isReady; private set => SetProperty(ref _isReady, value); }
    public bool IsPaused { get => _isPaused; private set => SetProperty(ref _isPaused, value); }
    public double PositionSeconds { get => _positionSeconds; private set => SetProperty(ref _positionSeconds, value); }
    public double DurationSeconds { get => _durationSeconds; private set => SetProperty(ref _durationSeconds, value); }
    public string? CurrentFile { get => _currentFile; private set => SetProperty(ref _currentFile, value); }

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
        try
        {
            await _host.StartAsync(new MpvHostOptions(), ct);
            _host.Ipc.PropertyChanged += OnPropertyChange;
            await _host.Ipc.ObservePropertyAsync("time-pos", ct);
            await _host.Ipc.ObservePropertyAsync("duration", ct);
            await _host.Ipc.ObservePropertyAsync("pause", ct);
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

    public async Task TogglePauseAsync()
    {
        if (_host is null) return;
        try
        {
            await _host.Ipc.SetPropertyAsync("pause", !IsPaused);
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Toggle pause failed");
        }
    }

    public async Task SeekAsync(double seconds)
    {
        if (_host is null) return;
        try
        {
            _suppressSeekFeedback = true;
            await _host.Ipc.SeekClampedAsync(seconds);
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
        if (_host is null) return;
        try
        {
            _host.Ipc.SetCutBounds(start, end);
            if (start is { } s) await _host.Ipc.SetPropertyAsync("ab-loop-a", s.TotalSeconds);
            else await _host.Ipc.SetPropertyAsync("ab-loop-a", "no");
            if (end is { } eEnd) await _host.Ipc.SetPropertyAsync("ab-loop-b", eEnd.TotalSeconds);
            else await _host.Ipc.SetPropertyAsync("ab-loop-b", "no");
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "ApplyCutBounds failed");
        }
    }

    private void OnPropertyChange(object? sender, MpvPropertyChange e)
    {
        Action apply = () =>
        {
            switch (e.Name)
            {
                case "time-pos" when e.Data is JsonElement t && t.ValueKind == JsonValueKind.Number:
                    if (!_suppressSeekFeedback) PositionSeconds = t.GetDouble();
                    break;
                case "duration" when e.Data is JsonElement d && d.ValueKind == JsonValueKind.Number:
                    DurationSeconds = d.GetDouble();
                    break;
                case "pause" when e.Data is JsonElement p:
                    IsPaused = p.ValueKind == JsonValueKind.True;
                    break;
            }
        };

        if (_uiContext is null) apply();
        else _uiContext.Post(_ => apply(), null);
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
