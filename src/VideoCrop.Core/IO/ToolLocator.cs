using System.Diagnostics.CodeAnalysis;

namespace VideoCrop.Core.IO;

public enum ExternalTool
{
    Ffmpeg,
    Ffprobe,
    Mpv,
}

public interface IToolLocator
{
    /// <summary>Directory the app manages for ffmpeg / ffprobe / mpv.</summary>
    string ToolsDirectory { get; }

    bool TryResolve(ExternalTool tool, [NotNullWhen(true)] out string? path);
    string GetExpectedPath(ExternalTool tool);
}

/// <summary>
/// Resolves external tools (ffmpeg, ffprobe, mpv) strictly from the app-managed
/// <c>tools/</c> directory next to the executable. We do not fall back to
/// <c>PATH</c> or any user-configured path: VideoCrop owns the tool versions
/// it depends on and downloads them itself on first run.
/// </summary>
public sealed class ToolLocator(string? toolsDirectory = null) : IToolLocator
{
    public string ToolsDirectory { get; } = toolsDirectory ?? Path.Combine(AppContext.BaseDirectory, "tools");

    public bool TryResolve(ExternalTool tool, [NotNullWhen(true)] out string? path)
    {
        var candidate = GetExpectedPath(tool);
        if (File.Exists(candidate))
        {
            path = candidate;
            return true;
        }
        path = null;
        return false;
    }

    public string GetExpectedPath(ExternalTool tool) =>
        Path.Combine(ToolsDirectory, ToFileName(tool));

    private static string ToFileName(ExternalTool tool) => tool switch
    {
        ExternalTool.Ffmpeg => OperatingSystem.IsWindows() ? "ffmpeg.exe" : "ffmpeg",
        ExternalTool.Ffprobe => OperatingSystem.IsWindows() ? "ffprobe.exe" : "ffprobe",
        ExternalTool.Mpv => OperatingSystem.IsWindows() ? "mpv.exe" : "mpv",
        _ => throw new ArgumentOutOfRangeException(nameof(tool)),
    };
}
