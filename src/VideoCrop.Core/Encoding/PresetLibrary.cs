using VideoCrop.Core.Models;

namespace VideoCrop.Core.Encoding;

public enum PresetCategory { Compatibility, Optimization, Custom }

public sealed record PresetDefinition(
    string Id,
    string DisplayName,
    PresetCategory Category,
    string Description,
    string? CompatibilityWarning,
    CompressionSpec Spec);

public static class PresetLibrary
{
    public static readonly PresetDefinition WebHigh = new(
        Id: "web-high",
        DisplayName: "High h264 (Compatibility)",
        Category: PresetCategory.Compatibility,
        Description: "Near-lossless. Best for archiving or re-editing.",
        CompatibilityWarning: null,
        Spec: new CompressionSpec(VideoCodec.H264, RateControl.Crf, 19, null, null, SpeedPreset.Slow, AudioCodec.Aac, 192, PixelFormat.Yuv420p));

    public static readonly PresetDefinition WebMedium = new(
        Id: "web-medium",
        DisplayName: "Medium h264 (Compatibility)",
        Category: PresetCategory.Compatibility,
        Description: "Default. Standard web quality; small files, plays everywhere.",
        CompatibilityWarning: null,
        Spec: new CompressionSpec(VideoCodec.H264, RateControl.Crf, 23, null, null, SpeedPreset.Medium, AudioCodec.Aac, 128, PixelFormat.Yuv420p));

    public static readonly PresetDefinition WebLow = new(
        Id: "web-low",
        DisplayName: "Low h264 (Compatibility)",
        Category: PresetCategory.Compatibility,
        Description: "Aggressive compression. Use for chat clips where size matters most.",
        CompatibilityWarning: null,
        Spec: new CompressionSpec(VideoCodec.H264, RateControl.Crf, 27, null, null, SpeedPreset.Medium, AudioCodec.Aac, 96, PixelFormat.Yuv420p));

    public static readonly PresetDefinition HighFast = new(
        Id: "high-fast",
        DisplayName: "High h265",
        Category: PresetCategory.Optimization,
        Description: "~40% smaller than H.264 High at similar quality.",
        CompatibilityWarning: "May not play in Discord embeds, older browsers, or older devices.",
        Spec: new CompressionSpec(VideoCodec.H265, RateControl.Crf, 23, null, null, SpeedPreset.Slow, AudioCodec.Opus, 160, PixelFormat.Yuv420p));

    public static readonly PresetDefinition HighSlow = new(
        Id: "high-slow",
        DisplayName: "High av1",
        Category: PresetCategory.Optimization,
        Description: "Best compression at this quality tier. Slower to encode.",
        CompatibilityWarning: "Limited compatibility (Safari, many chat apps, older hardware).",
        Spec: new CompressionSpec(VideoCodec.Av1, RateControl.Crf, 28, null, null, SpeedPreset.Slow, AudioCodec.Opus, 160, PixelFormat.Yuv420p));

    public static readonly PresetDefinition MediumFast = new(
        Id: "medium-fast",
        DisplayName: "Medium h265",
        Category: PresetCategory.Optimization,
        Description: "Smaller than Web: Medium with similar visual quality.",
        CompatibilityWarning: "Same H.265 caveats as above.",
        Spec: new CompressionSpec(VideoCodec.H265, RateControl.Crf, 27, null, null, SpeedPreset.Medium, AudioCodec.Opus, 128, PixelFormat.Yuv420p));

    public static readonly PresetDefinition MediumSlow = new(
        Id: "medium-slow",
        DisplayName: "Medium av1",
        Category: PresetCategory.Optimization,
        Description: "Smallest file at default quality.",
        CompatibilityWarning: "Same AV1 caveats as above.",
        Spec: new CompressionSpec(VideoCodec.Av1, RateControl.Crf, 32, null, null, SpeedPreset.Medium, AudioCodec.Opus, 128, PixelFormat.Yuv420p));

    public static readonly PresetDefinition LowFast = new(
        Id: "low-fast",
        DisplayName: "Low h265",
        Category: PresetCategory.Optimization,
        Description: "Tiny chat-clip files.",
        CompatibilityWarning: "Same H.265 caveats as above.",
        Spec: new CompressionSpec(VideoCodec.H265, RateControl.Crf, 30, null, null, SpeedPreset.Medium, AudioCodec.Opus, 96, PixelFormat.Yuv420p));

    public static readonly PresetDefinition LowSlow = new(
        Id: "low-slow",
        DisplayName: "Low av1",
        Category: PresetCategory.Optimization,
        Description: "Smallest possible at watchable quality.",
        CompatibilityWarning: "Same AV1 caveats as above.",
        Spec: new CompressionSpec(VideoCodec.Av1, RateControl.Crf, 36, null, null, SpeedPreset.Faster, AudioCodec.Opus, 96, PixelFormat.Yuv420p));

    public static readonly IReadOnlyList<PresetDefinition> All = new[]
    {
        WebHigh, WebMedium, WebLow,
        HighFast, HighSlow, MediumFast, MediumSlow, LowFast, LowSlow,
    };

    public static PresetDefinition GetById(string id) =>
        All.FirstOrDefault(p => p.Id == id) ?? WebMedium;
}
