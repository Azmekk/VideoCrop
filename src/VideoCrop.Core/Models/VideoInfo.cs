namespace VideoCrop.Core.Models;

public sealed record VideoInfo(
    string Path,
    TimeSpan Duration,
    int Width,
    int Height,
    int SampleAspectRatioNum,
    int SampleAspectRatioDen,
    string VideoCodec,
    double Fps,
    long? BitrateBitsPerSecond,
    IReadOnlyList<AudioStreamInfo> AudioStreams,
    string ContainerFormat)
{
    public double DisplayAspectRatio
    {
        get
        {
            if (Height <= 0) return 1.0;
            var sarNum = SampleAspectRatioNum > 0 ? SampleAspectRatioNum : 1;
            var sarDen = SampleAspectRatioDen > 0 ? SampleAspectRatioDen : 1;
            return (double)Width * sarNum / (Height * sarDen);
        }
    }
}

public sealed record AudioStreamInfo(
    int Index,
    string Codec,
    int? Channels,
    int? SampleRate,
    long? BitrateBitsPerSecond,
    string? Language);
