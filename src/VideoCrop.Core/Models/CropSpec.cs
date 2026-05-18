namespace VideoCrop.Core.Models;

public sealed record CropSpec(int X, int Y, int Width, int Height)
{
    public CropSpec Clamped(int sourceWidth, int sourceHeight)
    {
        var x = Math.Clamp(X, 0, Math.Max(0, sourceWidth - 1));
        var y = Math.Clamp(Y, 0, Math.Max(0, sourceHeight - 1));
        var maxW = sourceWidth - x;
        var maxH = sourceHeight - y;
        var w = Math.Clamp(Width, 1, Math.Max(1, maxW));
        var h = Math.Clamp(Height, 1, Math.Max(1, maxH));
        if ((w & 1) == 1) w--;
        if ((h & 1) == 1) h--;
        if ((x & 1) == 1) x++;
        if ((y & 1) == 1) y++;
        return new CropSpec(x, y, Math.Max(2, w), Math.Max(2, h));
    }

    public bool MatchesSource(int sourceWidth, int sourceHeight) =>
        X == 0 && Y == 0 && Width == sourceWidth && Height == sourceHeight;
}
