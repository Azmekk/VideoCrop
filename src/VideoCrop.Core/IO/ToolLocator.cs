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
    bool TryResolve(ExternalTool tool, [NotNullWhen(true)] out string? path);
    string? GetUserConfiguredPath(ExternalTool tool);
    void SetUserConfiguredPath(ExternalTool tool, string? path);
}

public sealed class ToolLocator(string? bundledToolsDir = null) : IToolLocator
{
    private readonly string _bundledToolsDir = bundledToolsDir ?? GetDefaultBundledToolsDir();
    private readonly Dictionary<ExternalTool, string?> _userPaths = new();

    public bool TryResolve(ExternalTool tool, [NotNullWhen(true)] out string? path)
    {
        var fileName = ToFileName(tool);

        var bundled = Path.Combine(_bundledToolsDir, fileName);
        if (File.Exists(bundled))
        {
            path = bundled;
            return true;
        }

        if (_userPaths.TryGetValue(tool, out var configured)
            && !string.IsNullOrWhiteSpace(configured)
            && File.Exists(configured))
        {
            path = configured;
            return true;
        }

        var onPath = FindOnPath(fileName);
        if (onPath != null)
        {
            path = onPath;
            return true;
        }

        path = null;
        return false;
    }

    public string? GetUserConfiguredPath(ExternalTool tool) =>
        _userPaths.TryGetValue(tool, out var p) ? p : null;

    public void SetUserConfiguredPath(ExternalTool tool, string? path) =>
        _userPaths[tool] = path;

    private static string ToFileName(ExternalTool tool) => tool switch
    {
        ExternalTool.Ffmpeg => OperatingSystem.IsWindows() ? "ffmpeg.exe" : "ffmpeg",
        ExternalTool.Ffprobe => OperatingSystem.IsWindows() ? "ffprobe.exe" : "ffprobe",
        ExternalTool.Mpv => OperatingSystem.IsWindows() ? "mpv.exe" : "mpv",
        _ => throw new ArgumentOutOfRangeException(nameof(tool)),
    };

    private static string GetDefaultBundledToolsDir()
    {
        var exeDir = AppContext.BaseDirectory;
        return Path.Combine(exeDir, "tools");
    }

    private static string? FindOnPath(string fileName)
    {
        var pathEnv = Environment.GetEnvironmentVariable("PATH");
        if (string.IsNullOrEmpty(pathEnv)) return null;

        var sep = OperatingSystem.IsWindows() ? ';' : ':';
        foreach (var dir in pathEnv.Split(sep, StringSplitOptions.RemoveEmptyEntries))
        {
            try
            {
                var candidate = Path.Combine(dir.Trim(), fileName);
                if (File.Exists(candidate)) return candidate;
            }
            catch
            {
                // Ignore malformed PATH entries.
            }
        }
        return null;
    }
}
