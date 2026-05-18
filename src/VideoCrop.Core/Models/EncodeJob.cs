namespace VideoCrop.Core.Models;

public enum OutputMode
{
    /// <summary>Both video and audio (first stream of each).</summary>
    Everything,
    /// <summary>Audio only, video discarded (<c>-vn</c>).</summary>
    AudioOnly,
    /// <summary>Video only, audio discarded (<c>-an</c>).</summary>
    VideoOnly,
}

public sealed record EncodeJob(
    string InputPath,
    string OutputPath,
    CutSpec? Cut,
    CropSpec? Crop,
    ResizeSpec? Resize,
    CompressionSpec? Compression,
    TimeSpan SourceDuration,
    OutputMode Mode = OutputMode.Everything);
