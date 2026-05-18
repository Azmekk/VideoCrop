namespace VideoCrop.Core.Models;

public sealed record ResizeSpec(int Width, int Height, bool AspectLocked)
{
    public static int RoundEven(int value) => value % 2 == 0 ? value : value - 1;

    public ResizeSpec WithEvenDimensions() => new(RoundEven(Width), RoundEven(Height), AspectLocked);
}
