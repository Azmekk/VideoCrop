using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.Extensions.Logging;
using VideoCrop.App.Services;
using VideoCrop.Core.Models;
using VideoCrop.Core.Processes;

namespace VideoCrop.App.ViewModels;

public sealed class MainViewModel : ObservableObject
{
    public ToolStatusViewModel ToolStatus { get; }
    public SourceViewModel Source { get; }
    public CompressionViewModel Compression { get; }
    public OutputViewModel Output { get; }
    public EncodeViewModel Encode { get; }
    public CutViewModel Cut { get; } = new();
    public CropViewModel Crop { get; } = new();
    public ResizeViewModel Resize { get; } = new();
    public UpdateViewModel Update { get; }
    public ToolSetupViewModel ToolSetup { get; }

    public MainViewModel(AppServices services)
    {
        ToolStatus = new ToolStatusViewModel(
            services.ToolLocator,
            services.LoggerFactory.CreateLogger<ToolStatusViewModel>());

        var ffprobe = new FfprobeRunner(
            services.ToolLocator,
            services.LoggerFactory.CreateLogger<FfprobeRunner>());

        Source = new SourceViewModel(
            ffprobe,
            services.LoggerFactory.CreateLogger<SourceViewModel>());

        Compression = new CompressionViewModel();
        Output = new OutputViewModel();

        var ffmpeg = new FfmpegRunner(
            services.ToolLocator,
            services.LoggerFactory.CreateLogger<FfmpegRunner>());

        Encode = new EncodeViewModel(
            ffmpeg,
            services.LoggerFactory.CreateLogger<EncodeViewModel>());

        Update = new UpdateViewModel(services.LoggerFactory);
        ToolSetup = new ToolSetupViewModel(services.ToolLocator, services.LoggerFactory);

        Source.VideoLoaded += (_, info) =>
        {
            Cut.SourceDuration = info.Duration;
            Cut.Reset();
            Crop.SetSource(info.Width, info.Height);
            Resize.SetSource(info.Width, info.Height);
            RefreshOutputBinding(info);
            OnPropertyChanged(nameof(PipelineSummary));
        };

        Crop.CropChanged += (_, _) =>
        {
            var toast = Resize.OnCropChanged(Crop.Crop, Crop.SourceWidth, Crop.SourceHeight);
            if (toast is not null) ResizeToast?.Invoke(this, toast);
            OnPropertyChanged(nameof(PipelineSummary));
        };

        Resize.PropertyChanged += (_, _) => OnPropertyChanged(nameof(PipelineSummary));

        Compression.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName is nameof(CompressionViewModel.Spec))
            {
                if (Source.VideoInfo is not null) RefreshOutputBinding(Source.VideoInfo);
            }
        };
    }

    public event EventHandler<string>? ResizeToast;

    public string PipelineSummary
    {
        get
        {
            if (Source.VideoInfo is null) return "";
            var info = Source.VideoInfo;
            var parts = new System.Text.StringBuilder();

            // When compression is disabled, the encoder stream-copies and any
            // crop/resize is ignored. Be explicit about that in the summary so
            // the user isn't surprised by leftover Crop/Resize panel state.
            if (!Compression.Enabled)
            {
                parts.Append("Output: stream copy (no re-encode)");
                if (Cut.AsSpecOrNull() is not null) parts.Append(", cut applied");
                if (Crop.Crop is not null || Resize.AsSpecOrNull() is not null)
                    parts.Append(" — crop/resize ignored");
                return parts.ToString();
            }

            var resize = Resize.AsSpecOrNull();
            var crop = Crop.Crop;
            int finalW = info.Width, finalH = info.Height;
            if (crop is not null) { finalW = crop.Width; finalH = crop.Height; }
            if (resize is not null)
            {
                var r = resize.WithEvenDimensions();
                if (r.AspectLocked && info.Height > 0)
                {
                    // Compute height from width using post-crop aspect.
                    var ar = (double)finalW / Math.Max(1, finalH);
                    finalH = (int)Math.Round(r.Width / ar);
                    finalW = r.Width;
                }
                else { finalW = r.Width; finalH = r.Height; }
            }
            var codecName = Compression.Spec.Codec switch
            {
                VideoCodec.H264 => "H.264",
                VideoCodec.H265 => "H.265",
                VideoCodec.Av1 => "AV1",
                VideoCodec.Vp9 => "VP9",
                _ => Compression.Spec.Codec.ToString(),
            };
            parts.Append($"Output: {finalW}×{finalH} {codecName}");
            if (crop is not null) parts.Append($" — crop {crop.Width}×{crop.Height} from {info.Width}×{info.Height}");
            if (resize is not null) parts.Append(crop is not null ? $", then scale to {finalW}×{finalH}" : $" — scale to {finalW}×{finalH}");
            return parts.ToString();
        }
    }

    private void RefreshOutputBinding(VideoInfo info)
    {
        Output.Bind(info.Path, Compression.Enabled ? Compression.Spec.Codec : null);
    }

    public async Task InitializeAsync()
    {
        await ToolStatus.RefreshAsync().ConfigureAwait(true);
        // Fire-and-forget the update check so startup isn't blocked by network.
        _ = Update.CheckAndStageAsync();
    }

    public EncodeJob? BuildEncodeJob()
    {
        if (Source.VideoInfo is null) return null;
        var outputPath = Output.BuildOutputPath();
        if (outputPath is null) return null;
        var compression = Compression.SpecOrNull;
        // Filters require re-encoding — stream-copy can't carry a -vf chain.
        var crop = compression is null ? null : Crop.Crop;
        var resize = compression is null ? null : Resize.AsSpecOrNull();
        return new EncodeJob(
            InputPath: Source.VideoInfo.Path,
            OutputPath: outputPath,
            Cut: Cut.AsSpecOrNull(),
            Crop: crop,
            Resize: resize,
            Compression: compression,
            SourceDuration: Source.VideoInfo.Duration);
    }
}
