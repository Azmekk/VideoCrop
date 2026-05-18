namespace VideoCrop.Core.Processes;

public sealed record ToolVersion(string Tool, string Version);

public static class ToolVersionProbe
{
    public static async Task<ToolVersion> GetFfmpegVersionAsync(string ffmpegPath, CancellationToken ct)
    {
        var output = await ExternalProcess.RunCaptureStdOutAsync(ffmpegPath, new[] { "-version" }, ct).ConfigureAwait(false);
        return new ToolVersion("ffmpeg", FirstLine(output));
    }

    public static async Task<ToolVersion> GetFfprobeVersionAsync(string ffprobePath, CancellationToken ct)
    {
        var output = await ExternalProcess.RunCaptureStdOutAsync(ffprobePath, new[] { "-version" }, ct).ConfigureAwait(false);
        return new ToolVersion("ffprobe", FirstLine(output));
    }

    public static async Task<ToolVersion> GetMpvVersionAsync(string mpvPath, CancellationToken ct)
    {
        var output = await ExternalProcess.RunCaptureStdOutAsync(mpvPath, new[] { "--version" }, ct).ConfigureAwait(false);
        return new ToolVersion("mpv", FirstLine(output));
    }

    private static string FirstLine(string output) =>
        output.Split('\n', 2, StringSplitOptions.RemoveEmptyEntries).FirstOrDefault()?.Trim() ?? "(unknown)";
}
