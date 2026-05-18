namespace VideoCrop.Core.Models;

public enum VideoCodec
{
    H264,
    H265,
    Av1,
    Vp9,
}

public enum AudioCodec
{
    Aac,
    Opus,
    Mp3,
    Copy,
}

public enum RateControl
{
    Crf,
    Cbr,
    Vbr,
    TwoPassTargetSize,
}

public enum SpeedPreset
{
    UltraFast,
    SuperFast,
    VeryFast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    VerySlow,
}

public enum PixelFormat
{
    Yuv420p,
    Yuv420p10le,
}

public static class CodecExtensions
{
    public static string ToFfmpegEncoder(this VideoCodec codec) => codec switch
    {
        VideoCodec.H264 => "libx264",
        VideoCodec.H265 => "libx265",
        VideoCodec.Av1 => "libsvtav1",
        VideoCodec.Vp9 => "libvpx-vp9",
        _ => throw new ArgumentOutOfRangeException(nameof(codec)),
    };

    public static string DefaultContainerExtension(this VideoCodec codec) => codec switch
    {
        VideoCodec.H264 or VideoCodec.H265 or VideoCodec.Av1 => ".mp4",
        VideoCodec.Vp9 => ".webm",
        _ => throw new ArgumentOutOfRangeException(nameof(codec)),
    };

    public static (int Min, int Max) CrfRange(this VideoCodec codec) => codec switch
    {
        VideoCodec.H264 or VideoCodec.H265 => (0, 51),
        VideoCodec.Av1 or VideoCodec.Vp9 => (0, 63),
        _ => (0, 51),
    };

    public static string ToFfmpegEncoder(this AudioCodec codec) => codec switch
    {
        AudioCodec.Aac => "aac",
        AudioCodec.Opus => "libopus",
        AudioCodec.Mp3 => "libmp3lame",
        AudioCodec.Copy => "copy",
        _ => throw new ArgumentOutOfRangeException(nameof(codec)),
    };

    public static string ToFfmpegPreset(this SpeedPreset preset, VideoCodec codec)
    {
        if (codec == VideoCodec.Av1)
        {
            return preset switch
            {
                SpeedPreset.UltraFast => "12",
                SpeedPreset.SuperFast => "11",
                SpeedPreset.VeryFast => "10",
                SpeedPreset.Faster => "9",
                SpeedPreset.Fast => "8",
                SpeedPreset.Medium => "6",
                SpeedPreset.Slow => "5",
                SpeedPreset.Slower => "4",
                SpeedPreset.VerySlow => "3",
                _ => "6",
            };
        }
        return preset switch
        {
            SpeedPreset.UltraFast => "ultrafast",
            SpeedPreset.SuperFast => "superfast",
            SpeedPreset.VeryFast => "veryfast",
            SpeedPreset.Faster => "faster",
            SpeedPreset.Fast => "fast",
            SpeedPreset.Medium => "medium",
            SpeedPreset.Slow => "slow",
            SpeedPreset.Slower => "slower",
            SpeedPreset.VerySlow => "veryslow",
            _ => "medium",
        };
    }

    public static string ToFfmpegPixelFormat(this PixelFormat pf) => pf switch
    {
        PixelFormat.Yuv420p => "yuv420p",
        PixelFormat.Yuv420p10le => "yuv420p10le",
        _ => "yuv420p",
    };
}
