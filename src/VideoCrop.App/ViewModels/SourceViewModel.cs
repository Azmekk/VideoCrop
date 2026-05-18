using System;
using System.Globalization;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.Extensions.Logging;
using VideoCrop.Core.Models;
using VideoCrop.Core.Processes;

namespace VideoCrop.App.ViewModels;

public sealed class SourceViewModel(IFfprobeRunner ffprobe, ILogger<SourceViewModel> logger) : ObservableObject
{
    private VideoInfo? _videoInfo;
    private string _statusMessage = "No video loaded.";
    private bool _isLoading;

    public VideoInfo? VideoInfo
    {
        get => _videoInfo;
        private set
        {
            if (SetProperty(ref _videoInfo, value))
            {
                OnPropertyChanged(nameof(HasVideo));
                OnPropertyChanged(nameof(FileName));
                OnPropertyChanged(nameof(DurationDisplay));
                OnPropertyChanged(nameof(ResolutionDisplay));
                OnPropertyChanged(nameof(CodecDisplay));
                OnPropertyChanged(nameof(BitrateDisplay));
                OnPropertyChanged(nameof(FpsDisplay));
                OnPropertyChanged(nameof(AudioDisplay));
            }
        }
    }

    public string StatusMessage { get => _statusMessage; private set => SetProperty(ref _statusMessage, value); }
    public bool IsLoading { get => _isLoading; private set => SetProperty(ref _isLoading, value); }

    public bool HasVideo => _videoInfo is not null;
    public string FileName => _videoInfo is null ? "" : Path.GetFileName(_videoInfo.Path);
    public string DurationDisplay => _videoInfo is null ? "" : FormatDuration(_videoInfo.Duration);
    public string ResolutionDisplay => _videoInfo is null
        ? ""
        : $"{_videoInfo.Width}×{_videoInfo.Height} ({_videoInfo.DisplayAspectRatio:0.##}:1)";
    public string CodecDisplay => _videoInfo?.VideoCodec ?? "";
    public string BitrateDisplay => _videoInfo?.BitrateBitsPerSecond is { } bps
        ? FormatBitrate(bps)
        : "";
    public string FpsDisplay => _videoInfo is null ? "" : _videoInfo.Fps.ToString("0.###", CultureInfo.InvariantCulture) + " fps";
    public string AudioDisplay => _videoInfo is null
        ? ""
        : _videoInfo.AudioStreams.Count == 0
            ? "no audio"
            : $"{_videoInfo.AudioStreams.Count} track(s), {_videoInfo.AudioStreams[0].Codec}";

    public event EventHandler<VideoInfo>? VideoLoaded;

    public async Task LoadAsync(string path, CancellationToken cancellationToken = default)
    {
        IsLoading = true;
        StatusMessage = $"Loading {Path.GetFileName(path)}…";
        try
        {
            var info = await ffprobe.GetVideoInfoAsync(path, cancellationToken).ConfigureAwait(true);
            VideoInfo = info;
            StatusMessage = "";
            VideoLoaded?.Invoke(this, info);
        }
        catch (Exception ex)
        {
            logger.LogError(ex, "Failed to load video {Path}", path);
            VideoInfo = null;
            StatusMessage = $"Failed to load: {ex.Message}";
        }
        finally
        {
            IsLoading = false;
        }
    }

    private static string FormatDuration(TimeSpan d)
    {
        if (d.TotalHours >= 1)
            return d.ToString(@"h\:mm\:ss\.fff", CultureInfo.InvariantCulture);
        return d.ToString(@"m\:ss\.fff", CultureInfo.InvariantCulture);
    }

    private static string FormatBitrate(long bps)
    {
        if (bps >= 1_000_000) return (bps / 1_000_000.0).ToString("0.##", CultureInfo.InvariantCulture) + " Mbps";
        if (bps >= 1_000) return (bps / 1_000.0).ToString("0", CultureInfo.InvariantCulture) + " kbps";
        return bps + " bps";
    }
}
