using System;
using System.Threading;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.Extensions.Logging;
using VideoCrop.Core.IO;
using VideoCrop.Core.Processes;

namespace VideoCrop.App.ViewModels;

public sealed class ToolStatusViewModel(IToolLocator locator, ILogger<ToolStatusViewModel> logger) : ObservableObject
{
    private string _ffmpegStatus = "checking…";
    private string _ffprobeStatus = "checking…";
    private string _mpvStatus = "checking…";
    private bool _allFound;

    public string FfmpegStatus { get => _ffmpegStatus; private set => SetProperty(ref _ffmpegStatus, value); }
    public string FfprobeStatus { get => _ffprobeStatus; private set => SetProperty(ref _ffprobeStatus, value); }
    public string MpvStatus { get => _mpvStatus; private set => SetProperty(ref _mpvStatus, value); }
    public bool AllFound { get => _allFound; private set => SetProperty(ref _allFound, value); }

    public async Task RefreshAsync(CancellationToken ct = default)
    {
        FfmpegStatus = await ProbeAsync(ExternalTool.Ffmpeg, ct).ConfigureAwait(true);
        FfprobeStatus = await ProbeAsync(ExternalTool.Ffprobe, ct).ConfigureAwait(true);
        MpvStatus = await ProbeAsync(ExternalTool.Mpv, ct).ConfigureAwait(true);

        AllFound = locator.TryResolve(ExternalTool.Ffmpeg, out _)
                   && locator.TryResolve(ExternalTool.Ffprobe, out _)
                   && locator.TryResolve(ExternalTool.Mpv, out _);
    }

    private async Task<string> ProbeAsync(ExternalTool tool, CancellationToken ct)
    {
        if (!locator.TryResolve(tool, out var path))
            return "not found";

        try
        {
            var version = tool switch
            {
                ExternalTool.Ffmpeg => await ToolVersionProbe.GetFfmpegVersionAsync(path, ct).ConfigureAwait(true),
                ExternalTool.Ffprobe => await ToolVersionProbe.GetFfprobeVersionAsync(path, ct).ConfigureAwait(true),
                ExternalTool.Mpv => await ToolVersionProbe.GetMpvVersionAsync(path, ct).ConfigureAwait(true),
                _ => new ToolVersion(tool.ToString(), "?"),
            };
            return version.Version;
        }
        catch (Exception ex)
        {
            logger.LogWarning(ex, "Failed to probe {Tool}", tool);
            return "found but probe failed";
        }
    }
}
