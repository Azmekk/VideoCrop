using System;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.Extensions.Logging;
using VideoCrop.Core.IO;

namespace VideoCrop.App.ViewModels;

public sealed class ToolSetupViewModel(IToolLocator locator, ILoggerFactory loggerFactory) : ObservableObject
{
    private static readonly HttpClient SharedHttp = new(new HttpClientHandler { AllowAutoRedirect = true })
    {
        Timeout = TimeSpan.FromMinutes(10),
    };

    private readonly ILogger<ToolSetupViewModel> _logger = loggerFactory.CreateLogger<ToolSetupViewModel>();

    private double _ffmpegProgress;
    private double _mpvProgress;
    private string _ffmpegStatus = "Pending";
    private string _mpvStatus = "Pending";
    private bool _isDownloading;
    private string? _errorMessage;

    public double FfmpegProgress { get => _ffmpegProgress; private set => SetProperty(ref _ffmpegProgress, value); }
    public double MpvProgress { get => _mpvProgress; private set => SetProperty(ref _mpvProgress, value); }
    public string FfmpegStatus { get => _ffmpegStatus; private set => SetProperty(ref _ffmpegStatus, value); }
    public string MpvStatus { get => _mpvStatus; private set => SetProperty(ref _mpvStatus, value); }
    public bool IsDownloading { get => _isDownloading; private set => SetProperty(ref _isDownloading, value); }
    public string? ErrorMessage { get => _errorMessage; private set => SetProperty(ref _errorMessage, value); }

    public bool ToolsMissing =>
        !locator.TryResolve(ExternalTool.Ffmpeg, out _)
        || !locator.TryResolve(ExternalTool.Ffprobe, out _)
        || !locator.TryResolve(ExternalTool.Mpv, out _);

    public async Task<bool> DownloadAsync(CancellationToken ct = default)
    {
        if (IsDownloading) return false;
        IsDownloading = true;
        ErrorMessage = null;
        FfmpegProgress = 0; MpvProgress = 0;
        FfmpegStatus = "Queued"; MpvStatus = "Queued";

        try
        {
            var service = new ToolDownloadService(
                locator,
                SharedHttp,
                loggerFactory.CreateLogger<ToolDownloadService>());

            var progress = new Progress<ToolDownloadProgress>(p =>
            {
                if (p.Tool == "ffmpeg")
                {
                    FfmpegProgress = p.Fraction;
                    FfmpegStatus = p.Status;
                }
                else if (p.Tool == "mpv")
                {
                    MpvProgress = p.Fraction;
                    MpvStatus = p.Status;
                }
            });

            var ok = await service.DownloadAllAsync(progress, ct);
            if (!ok)
            {
                ErrorMessage = "Download failed. Check your network connection and try again.";
                return false;
            }
            return true;
        }
        catch (OperationCanceledException)
        {
            ErrorMessage = "Cancelled.";
            return false;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Tool download error");
            ErrorMessage = $"Download failed: {ex.Message}";
            return false;
        }
        finally
        {
            IsDownloading = false;
        }
    }
}
