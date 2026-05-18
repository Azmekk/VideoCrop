using System;
using System.Diagnostics;
using System.IO;
using System.Net.Http;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.Extensions.Logging;
using VideoCrop.App.Updater;
using VideoCrop.Core.Updater;

namespace VideoCrop.App.ViewModels;

public sealed class UpdateViewModel(ILoggerFactory loggerFactory) : ObservableObject
{
    private readonly ILogger<UpdateViewModel> _logger = loggerFactory.CreateLogger<UpdateViewModel>();

    private static readonly HttpClient SharedHttp = new(new HttpClientHandler
    {
        AllowAutoRedirect = true,
    })
    { Timeout = TimeSpan.FromSeconds(60) };

    private UpdateState _state = UpdateState.Idle;
    private UpdateInfo? _info;
    private double _downloadProgress;
    private string _statusMessage = "";
    private string? _stagedDir;

    public UpdateState State { get => _state; private set { if (SetProperty(ref _state, value)) OnPropertyChanged(nameof(IsActionable)); } }
    public UpdateInfo? Info { get => _info; private set => SetProperty(ref _info, value); }
    public double DownloadProgress { get => _downloadProgress; private set => SetProperty(ref _downloadProgress, value); }
    public string StatusMessage { get => _statusMessage; private set => SetProperty(ref _statusMessage, value); }

    /// <summary>True when the bar should be shown to the user.</summary>
    public bool IsActionable => _state is UpdateState.UpdateAvailable
        or UpdateState.Downloading
        or UpdateState.Staged
        or UpdateState.Failed;

    public static Version CurrentVersion =>
        Assembly.GetExecutingAssembly().GetName().Version ?? new Version(0, 0, 0, 0);

    public async Task CheckAndStageAsync(CancellationToken ct = default)
    {
        try
        {
            State = UpdateState.Checking;
            StatusMessage = "Checking for updates…";

            var installDir = AppContext.BaseDirectory;
            var stagingRoot = Path.Combine(installDir, "update");
            var service = new UpdateService(
                SharedHttp,
                AppMetadata.GitHubRepo,
                stagingRoot,
                loggerFactory.CreateLogger<UpdateService>());

            var info = await service.CheckForUpdateAsync(CurrentVersion, ct);
            if (info is null)
            {
                State = UpdateState.UpToDate;
                StatusMessage = "";
                return;
            }

            Info = info;
            State = UpdateState.UpdateAvailable;
            StatusMessage = $"Update available: {info.TagName}";
            _logger.LogInformation("Update available: {Tag}", info.TagName);

            // Auto-download in background so "Restart" is immediate.
            State = UpdateState.Downloading;
            StatusMessage = "Downloading update…";
            var progress = new Progress<double>(p =>
            {
                DownloadProgress = p;
                StatusMessage = $"Downloading update… {p * 100:0}%";
            });
            var stagedDir = await service.DownloadAndStageAsync(info, progress, ct);
            if (stagedDir is null)
            {
                State = UpdateState.Failed;
                StatusMessage = "Update download failed. See log for details.";
                return;
            }
            _stagedDir = stagedDir;
            State = UpdateState.Staged;
            StatusMessage = $"Update {info.TagName} ready — restart to apply.";
        }
        catch (OperationCanceledException)
        {
            State = UpdateState.Idle;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Update flow failed");
            State = UpdateState.Failed;
            StatusMessage = "Update check failed.";
        }
    }

    public bool TryApplyAndRestart()
    {
        if (State != UpdateState.Staged || _stagedDir is null) return false;
        try
        {
            State = UpdateState.Applying;
            StatusMessage = "Applying update…";
            var installDir = AppContext.BaseDirectory.TrimEnd(Path.DirectorySeparatorChar);
            var relaunchExe = Path.Combine(installDir, "VideoCrop.App.exe");
            UpdateApplier.ApplyAndRestart(installDir, _stagedDir, Process.GetCurrentProcess().Id, relaunchExe);

            // Exit immediately so the helper can swap files. Application.Exit
            // tears down the WinUI window; the PowerShell helper is already
            // waiting on our PID and will take over.
            Microsoft.UI.Xaml.Application.Current.Exit();
            return true;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Apply update failed");
            State = UpdateState.Failed;
            StatusMessage = "Failed to apply update.";
            return false;
        }
    }
}
