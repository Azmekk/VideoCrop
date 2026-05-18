using System.Globalization;
using System.Text.Json;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;
using VideoCrop.Core.IO;
using VideoCrop.Core.Models;
using VideoCrop.Core.Serialization;

namespace VideoCrop.Core.Processes;

public interface IFfprobeRunner
{
    Task<VideoInfo> GetVideoInfoAsync(string inputPath, CancellationToken cancellationToken);
}

public sealed class FfprobeRunner(IToolLocator locator, ILogger<FfprobeRunner>? logger = null) : IFfprobeRunner
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNameCaseInsensitive = true,
    };

    private readonly ILogger<FfprobeRunner> _logger = logger ?? NullLogger<FfprobeRunner>.Instance;

    public async Task<VideoInfo> GetVideoInfoAsync(string inputPath, CancellationToken cancellationToken)
    {
        if (!locator.TryResolve(ExternalTool.Ffprobe, out var ffprobe))
            throw new InvalidOperationException("ffprobe not found.");

        var args = new[]
        {
            "-v", "error",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            inputPath,
        };

        var json = await ExternalProcess.RunCaptureStdOutAsync(ffprobe, args, cancellationToken).ConfigureAwait(false);

        try
        {
            return ParseVideoInfo(json, inputPath);
        }
        catch (JsonException ex)
        {
            _logger.LogError(ex, "Failed to parse ffprobe output for {Path}", inputPath);
            throw new InvalidDataException("ffprobe returned unparseable JSON.", ex);
        }
    }

    internal static VideoInfo ParseVideoInfo(string json, string inputPath)
    {
        var parsed = JsonSerializer.Deserialize<FfprobeOutput>(json, JsonOptions)
            ?? throw new InvalidDataException("ffprobe returned empty output.");

        var videoStream = parsed.Streams.FirstOrDefault(s =>
            string.Equals(s.CodecType, "video", StringComparison.OrdinalIgnoreCase))
            ?? throw new InvalidDataException("Input has no video stream.");

        var duration = ParseDuration(parsed.Format.Duration, videoStream.Duration);
        var (sarNum, sarDen) = ParseRatio(videoStream.SampleAspectRatio, 1, 1);
        var fps = ParseRate(videoStream.AvgFrameRate ?? videoStream.RFrameRate);
        var bitrate = ParseLong(parsed.Format.BitRate);

        var audio = parsed.Streams
            .Where(s => string.Equals(s.CodecType, "audio", StringComparison.OrdinalIgnoreCase))
            .Select(s => new AudioStreamInfo(
                Index: s.Index,
                Codec: s.CodecName ?? "?",
                Channels: s.Channels,
                SampleRate: int.TryParse(s.SampleRate, NumberStyles.Integer, CultureInfo.InvariantCulture, out var sr) ? sr : null,
                BitrateBitsPerSecond: ParseLong(s.BitRate),
                Language: s.Tags is not null && s.Tags.TryGetValue("language", out var lang) ? lang : null))
            .ToList();

        return new VideoInfo(
            Path: inputPath,
            Duration: duration,
            Width: videoStream.Width ?? 0,
            Height: videoStream.Height ?? 0,
            SampleAspectRatioNum: sarNum,
            SampleAspectRatioDen: sarDen,
            VideoCodec: videoStream.CodecName ?? "?",
            Fps: fps,
            BitrateBitsPerSecond: bitrate,
            AudioStreams: audio,
            ContainerFormat: parsed.Format.FormatName ?? "");
    }

    private static TimeSpan ParseDuration(string? formatDuration, string? streamDuration)
    {
        var s = formatDuration ?? streamDuration;
        if (string.IsNullOrEmpty(s)) return TimeSpan.Zero;
        if (double.TryParse(s, NumberStyles.Float, CultureInfo.InvariantCulture, out var seconds))
            return TimeSpan.FromSeconds(seconds);
        return TimeSpan.Zero;
    }

    private static (int Num, int Den) ParseRatio(string? raw, int fallbackNum, int fallbackDen)
    {
        if (string.IsNullOrEmpty(raw)) return (fallbackNum, fallbackDen);
        var parts = raw.Split(':', 2);
        if (parts.Length != 2) return (fallbackNum, fallbackDen);
        if (!int.TryParse(parts[0], NumberStyles.Integer, CultureInfo.InvariantCulture, out var n)) return (fallbackNum, fallbackDen);
        if (!int.TryParse(parts[1], NumberStyles.Integer, CultureInfo.InvariantCulture, out var d) || d == 0) return (fallbackNum, fallbackDen);
        return (n, d);
    }

    private static double ParseRate(string? raw)
    {
        if (string.IsNullOrEmpty(raw)) return 0;
        var parts = raw.Split('/', 2);
        if (parts.Length == 1)
            return double.TryParse(parts[0], NumberStyles.Float, CultureInfo.InvariantCulture, out var single) ? single : 0;
        if (!double.TryParse(parts[0], NumberStyles.Float, CultureInfo.InvariantCulture, out var num)) return 0;
        if (!double.TryParse(parts[1], NumberStyles.Float, CultureInfo.InvariantCulture, out var den) || den == 0) return 0;
        return num / den;
    }

    private static long? ParseLong(string? raw)
    {
        if (string.IsNullOrEmpty(raw)) return null;
        return long.TryParse(raw, NumberStyles.Integer, CultureInfo.InvariantCulture, out var v) ? v : null;
    }
}
