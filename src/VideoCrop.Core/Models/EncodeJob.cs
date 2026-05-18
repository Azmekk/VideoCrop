namespace VideoCrop.Core.Models;

public sealed record EncodeJob(
    string InputPath,
    string OutputPath,
    CutSpec? Cut,
    CropSpec? Crop,
    ResizeSpec? Resize,
    CompressionSpec? Compression,
    TimeSpan SourceDuration);
