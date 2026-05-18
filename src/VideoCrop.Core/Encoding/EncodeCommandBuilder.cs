using System.Globalization;
using VideoCrop.Core.Models;

namespace VideoCrop.Core.Encoding;

public sealed record FfmpegInvocation(IReadOnlyList<string> Arguments, bool TwoPass);

public sealed record TwoPassInvocations(IReadOnlyList<string> Pass1, IReadOnlyList<string> Pass2);

public static class EncodeCommandBuilder
{
    public static FfmpegInvocation Build(EncodeJob job)
    {
        var args = new List<string> { "-y", "-hide_banner", "-nostats" };

        var preInput = job.Cut is not null && !job.Cut.Accurate;
        if (preInput && job.Cut is not null)
        {
            args.Add("-ss"); args.Add(FormatSeconds(job.Cut.Start));
            args.Add("-to"); args.Add(FormatSeconds(job.Cut.End));
        }

        args.Add("-i"); args.Add(job.InputPath);

        if (job.Cut is not null && job.Cut.Accurate)
        {
            args.Add("-ss"); args.Add(FormatSeconds(job.Cut.Start));
            args.Add("-to"); args.Add(FormatSeconds(job.Cut.End));
        }

        var wantVideo = job.Mode != OutputMode.AudioOnly;
        var wantAudio = job.Mode != OutputMode.VideoOnly;

        // When Compression is null, the user explicitly turned re-encoding off:
        // stream-copy whatever streams the mode wants. -vn/-an drop the
        // disabled track entirely.
        if (job.Compression is null)
        {
            if (wantVideo) { args.Add("-c:v"); args.Add("copy"); }
            else           { args.Add("-vn"); }
            if (wantAudio) { args.Add("-c:a"); args.Add("copy"); }
            else           { args.Add("-an"); }
        }
        else
        {
            if (wantVideo)
            {
                var filter = BuildVideoFilter(job);
                if (filter is not null)
                {
                    args.Add("-vf"); args.Add(filter);
                }
                AddVideoEncoderArgs(args, job.Compression);
            }
            else
            {
                args.Add("-vn");
            }

            if (wantAudio) AddAudioArgs(args, job.Compression);
            else           args.Add("-an");
        }

        // Map the streams the mode wants, in source order.
        if (wantVideo) { args.Add("-map"); args.Add("0:v:0"); }
        if (wantAudio) { args.Add("-map"); args.Add("0:a:0?"); }
        args.Add("-map_metadata"); args.Add("-1");

        args.Add("-progress"); args.Add("pipe:1");
        args.Add("-nostats");

        args.Add(job.OutputPath);

        return new FfmpegInvocation(args, TwoPass: false);
    }

    public static TwoPassInvocations BuildTwoPass(EncodeJob job, string statsFilePrefix)
    {
        var comp = job.Compression
            ?? throw new InvalidOperationException("Two-pass requires a CompressionSpec.");
        if (comp.Codec is not (VideoCodec.H264 or VideoCodec.H265 or VideoCodec.Vp9))
            throw new InvalidOperationException("Two-pass currently supports H.264, H.265, and VP9 only.");
        if (comp.TargetBitrateBps is not { } targetBps)
            throw new InvalidOperationException("Two-pass requires a target bitrate.");

        var nullSink = OperatingSystem.IsWindows() ? "NUL" : "/dev/null";

        var pass1 = new List<string> { "-y", "-hide_banner", "-nostats" };
        AddCutBeforeInput(pass1, job);
        pass1.Add("-i"); pass1.Add(job.InputPath);
        AddCutAfterInput(pass1, job);

        var filter = BuildVideoFilter(job);
        if (filter is not null) { pass1.Add("-vf"); pass1.Add(filter); }

        AddVideoEncoderArgs(pass1, comp, twoPassBitrate: targetBps);
        pass1.Add("-pass"); pass1.Add("1");
        pass1.Add("-passlogfile"); pass1.Add(statsFilePrefix);
        pass1.Add("-an");
        pass1.Add("-f"); pass1.Add("mp4");
        pass1.Add("-map"); pass1.Add("0:v:0");
        pass1.Add(nullSink);

        var pass2 = new List<string> { "-y", "-hide_banner", "-nostats" };
        AddCutBeforeInput(pass2, job);
        pass2.Add("-i"); pass2.Add(job.InputPath);
        AddCutAfterInput(pass2, job);
        if (filter is not null) { pass2.Add("-vf"); pass2.Add(filter); }
        AddVideoEncoderArgs(pass2, comp, twoPassBitrate: targetBps);
        pass2.Add("-pass"); pass2.Add("2");
        pass2.Add("-passlogfile"); pass2.Add(statsFilePrefix);
        AddAudioArgs(pass2, comp);
        pass2.Add("-map"); pass2.Add("0:v:0");
        pass2.Add("-map"); pass2.Add("0:a:0?");
        pass2.Add("-map_metadata"); pass2.Add("-1");
        pass2.Add("-progress"); pass2.Add("pipe:1");
        pass2.Add("-nostats");
        pass2.Add(job.OutputPath);

        return new TwoPassInvocations(pass1, pass2);
    }

    public static string? BuildVideoFilter(EncodeJob job)
    {
        var parts = new List<string>();

        if (job.Crop is not null)
        {
            var c = job.Crop;
            parts.Add($"crop={c.Width}:{c.Height}:{c.X}:{c.Y}");
        }

        if (job.Resize is not null)
        {
            var r = job.Resize.WithEvenDimensions();
            string scale;
            if (r.AspectLocked)
            {
                scale = $"scale={r.Width}:-2:flags=lanczos";
            }
            else
            {
                scale = $"scale={r.Width}:{r.Height}:flags=lanczos";
            }
            parts.Add(scale);
        }

        if (parts.Count == 0) return null;
        return string.Join(',', parts);
    }

    public static long ComputeTargetBitrate(long targetBytes, TimeSpan duration, int audioKbps, double safetyMargin = 0.97)
    {
        if (duration.TotalSeconds <= 0) throw new ArgumentException("Duration must be positive.", nameof(duration));
        var audioBitsPerSec = (long)audioKbps * 1000;
        var totalBits = (long)(targetBytes * 8 * safetyMargin);
        var audioBits = (long)(audioBitsPerSec * duration.TotalSeconds);
        var videoBits = totalBits - audioBits;
        var videoBps = (long)(videoBits / duration.TotalSeconds);
        return Math.Max(50_000, videoBps);
    }

    private static void AddCutBeforeInput(List<string> args, EncodeJob job)
    {
        if (job.Cut is not null && !job.Cut.Accurate)
        {
            args.Add("-ss"); args.Add(FormatSeconds(job.Cut.Start));
            args.Add("-to"); args.Add(FormatSeconds(job.Cut.End));
        }
    }

    private static void AddCutAfterInput(List<string> args, EncodeJob job)
    {
        if (job.Cut is not null && job.Cut.Accurate)
        {
            args.Add("-ss"); args.Add(FormatSeconds(job.Cut.Start));
            args.Add("-to"); args.Add(FormatSeconds(job.Cut.End));
        }
    }

    private static void AddVideoEncoderArgs(List<string> args, CompressionSpec c, long? twoPassBitrate = null)
    {
        args.Add("-c:v"); args.Add(c.Codec.ToFfmpegEncoder());

        if (c.Codec == VideoCodec.Av1)
        {
            args.Add("-preset"); args.Add(c.Speed.ToFfmpegPreset(c.Codec));
        }
        else if (c.Codec != VideoCodec.Vp9)
        {
            args.Add("-preset"); args.Add(c.Speed.ToFfmpegPreset(c.Codec));
        }

        if (twoPassBitrate is not null)
        {
            args.Add("-b:v"); args.Add(twoPassBitrate.Value.ToString(CultureInfo.InvariantCulture));
        }
        else
        {
            switch (c.RateControl)
            {
                case RateControl.Crf:
                    if (c.Codec is VideoCodec.Av1)
                    {
                        args.Add("-crf"); args.Add(c.Crf.ToString(CultureInfo.InvariantCulture));
                        args.Add("-b:v"); args.Add("0");
                    }
                    else if (c.Codec is VideoCodec.Vp9)
                    {
                        args.Add("-crf"); args.Add(c.Crf.ToString(CultureInfo.InvariantCulture));
                        args.Add("-b:v"); args.Add("0");
                    }
                    else
                    {
                        args.Add("-crf"); args.Add(c.Crf.ToString(CultureInfo.InvariantCulture));
                    }
                    break;
                case RateControl.Cbr:
                case RateControl.Vbr:
                    if (c.TargetBitrateBps is { } bps)
                    {
                        args.Add("-b:v"); args.Add(bps.ToString(CultureInfo.InvariantCulture));
                        if (c.RateControl == RateControl.Cbr)
                        {
                            args.Add("-maxrate"); args.Add(bps.ToString(CultureInfo.InvariantCulture));
                            args.Add("-minrate"); args.Add(bps.ToString(CultureInfo.InvariantCulture));
                            args.Add("-bufsize"); args.Add(((long)(bps * 2)).ToString(CultureInfo.InvariantCulture));
                        }
                    }
                    break;
                case RateControl.TwoPassTargetSize:
                    // Caller should use BuildTwoPass instead.
                    throw new InvalidOperationException("Use BuildTwoPass for target-size encoding.");
            }
        }

        args.Add("-pix_fmt"); args.Add(c.PixelFormat.ToFfmpegPixelFormat());
    }

    private static void AddAudioArgs(List<string> args, CompressionSpec c)
    {
        args.Add("-c:a"); args.Add(c.AudioCodec.ToFfmpegEncoder());
        if (c.AudioCodec != AudioCodec.Copy)
        {
            args.Add("-b:a"); args.Add($"{c.AudioBitrateKbps}k");
        }
    }

    private static string FormatSeconds(TimeSpan ts) =>
        ts.TotalSeconds.ToString("0.###", CultureInfo.InvariantCulture);
}
