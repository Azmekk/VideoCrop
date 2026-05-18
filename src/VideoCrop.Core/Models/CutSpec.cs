namespace VideoCrop.Core.Models;

public sealed record CutSpec(TimeSpan Start, TimeSpan End, bool Accurate)
{
    public TimeSpan Duration => End - Start;

    public static CutSpec FullRange(TimeSpan duration) => new(TimeSpan.Zero, duration, Accurate: false);

    public bool IsFullRange(TimeSpan sourceDuration)
    {
        return Start <= TimeSpan.Zero && End >= sourceDuration - TimeSpan.FromMilliseconds(1);
    }
}
