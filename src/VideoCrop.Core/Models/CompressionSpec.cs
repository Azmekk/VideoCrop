namespace VideoCrop.Core.Models;

public sealed record CompressionSpec(
    VideoCodec Codec,
    RateControl RateControl,
    int Crf,
    long? TargetBitrateBps,
    long? TargetSizeBytes,
    SpeedPreset Speed,
    AudioCodec AudioCodec,
    int AudioBitrateKbps,
    PixelFormat PixelFormat,
    string? ContainerExtensionOverride = null)
{
    public string ContainerExtension => ContainerExtensionOverride ?? Codec.DefaultContainerExtension();

    public static CompressionSpec WebMediumDefault() => new(
        Codec: VideoCodec.H264,
        RateControl: RateControl.Crf,
        Crf: 23,
        TargetBitrateBps: null,
        TargetSizeBytes: null,
        Speed: SpeedPreset.Medium,
        AudioCodec: AudioCodec.Aac,
        AudioBitrateKbps: 128,
        PixelFormat: PixelFormat.Yuv420p);
}
