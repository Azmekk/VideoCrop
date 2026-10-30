//! Quality presets (port of PresetLibrary.cs) and share-target platform presets.

use super::models::*;

pub const DEFAULT_QUALITY_ID: &str = "web-medium";

/// Decimal megabytes: upload limits are advertised in MB, and 10 000 000 bytes is safely
/// under both the 10 MB and 10 MiB readings.
pub const MB: u64 = 1_000_000;

const H265_WARNING: &str = "May not play in Discord embeds, older browsers, or older devices.";
const AV1_WARNING: &str = "Limited compatibility (Safari, many chat apps, older hardware).";

fn crf_spec(codec: VideoCodec, crf: u32, speed: SpeedPreset, audio: AudioCodec, audio_kbps: u32) -> CompressionSpec {
    CompressionSpec {
        codec,
        rate_control: RateControl::Crf,
        crf,
        target_bitrate_bps: None,
        target_size_bytes: None,
        speed,
        audio_codec: audio,
        audio_bitrate_kbps: audio_kbps,
        pixel_format: PixelFormat::Yuv420p,
        container: None,
    }
}

fn quality(id: &str, name: &str, category: PresetCategory, description: &str, warning: Option<&str>, spec: CompressionSpec) -> QualityPreset {
    QualityPreset {
        id: id.into(),
        name: name.into(),
        category,
        description: description.into(),
        warning: warning.map(str::to_string),
        spec,
    }
}

pub fn quality_presets() -> Vec<QualityPreset> {
    use AudioCodec::*;
    use PresetCategory::*;
    use SpeedPreset::*;
    use VideoCodec::*;
    vec![
        quality("web-high", "High h264 (Compatibility)", Compatibility,
            "Near-lossless. Best for archiving or re-editing.", None,
            crf_spec(H264, 19, Slow, Aac, 192)),
        quality("web-medium", "Medium h264 (Compatibility)", Compatibility,
            "Default. Standard web quality; small files, plays everywhere.", None,
            crf_spec(H264, 23, Medium, Aac, 128)),
        quality("web-low", "Low h264 (Compatibility)", Compatibility,
            "Aggressive compression. Use for chat clips where size matters most.", None,
            crf_spec(H264, 27, Medium, Aac, 96)),
        quality("high-fast", "High h265", Optimization,
            "~40% smaller than H.264 High at similar quality.", Some(H265_WARNING),
            crf_spec(H265, 23, Slow, Opus, 160)),
        quality("high-slow", "High av1", Optimization,
            "Best compression at this quality tier. Slower to encode.", Some(AV1_WARNING),
            crf_spec(Av1, 28, Slow, Opus, 160)),
        quality("medium-fast", "Medium h265", Optimization,
            "Smaller than Web: Medium with similar visual quality.", Some(H265_WARNING),
            crf_spec(H265, 27, Medium, Opus, 128)),
        quality("medium-slow", "Medium av1", Optimization,
            "Smallest file at default quality.", Some(AV1_WARNING),
            crf_spec(Av1, 32, Medium, Opus, 128)),
        quality("low-fast", "Low h265", Optimization,
            "Tiny chat-clip files.", Some(H265_WARNING),
            crf_spec(H265, 30, Medium, Opus, 96)),
        quality("low-slow", "Low av1", Optimization,
            "Smallest possible at watchable quality.", Some(AV1_WARNING),
            crf_spec(Av1, 36, Faster, Opus, 96)),
    ]
}

#[allow(clippy::too_many_arguments)]
fn platform(
    id: &str,
    name: &str,
    icon: &str,
    description: &str,
    cap_mb: Option<u64>,
    box_wh: Option<(u32, u32)>,
    crf: u32,
    speed: SpeedPreset,
    audio_kbps: u32,
) -> PlatformPreset {
    let max_size_bytes = cap_mb.map(|m| m * MB);
    let mut spec = crf_spec(VideoCodec::H264, crf, speed, AudioCodec::Aac, audio_kbps);
    if let Some(cap) = max_size_bytes {
        spec.rate_control = RateControl::TargetSize;
        spec.target_size_bytes = Some(cap);
    }
    PlatformPreset {
        id: id.into(),
        name: name.into(),
        icon: icon.into(),
        description: description.into(),
        max_size_bytes,
        max_width: box_wh.map(|b| b.0),
        max_height: box_wh.map(|b| b.1),
        spec,
    }
}

/// Share-target presets. Capped presets carry `RateControl::TargetSize` with
/// `target_size_bytes = cap`; the planner fills in `target_bitrate_bps` per job.
/// `spec.target_bitrate_bps` on a platform base spec, when set, is an extra absolute
/// bitrate ceiling on top of the planner's per-resolution ceiling (none set by default).
pub fn platform_presets() -> Vec<PlatformPreset> {
    use SpeedPreset::*;
    vec![
        platform("discord", "Discord", "discord", "Fits the 10 MB free upload limit.", Some(10), Some((1920, 1080)), 23, Medium, 128),
        platform("discord-nitro-basic", "Discord Nitro Basic", "discord", "Fits the 50 MB Nitro Basic upload limit.", Some(50), Some((1920, 1080)), 23, Medium, 128),
        platform("discord-nitro", "Discord Nitro", "discord", "Fits the 500 MB Nitro upload limit, up to 1440p.", Some(500), Some((2560, 1440)), 23, Medium, 160),
        platform("youtube", "YouTube", "youtube", "High-quality upload master; YouTube re-encodes anyway.", None, None, 18, Slow, 320),
        platform("x", "X / Twitter", "x", "Fits the 512 MB upload limit at up to 1920×1200.", Some(512), Some((1920, 1200)), 23, Medium, 128),
        platform("vertical", "TikTok / Reels / Shorts", "vertical", "Vertical 1080×1920 for short-form apps.", None, Some((1080, 1920)), 21, Medium, 128),
        platform("whatsapp", "WhatsApp", "whatsapp", "Fits the 16 MB video limit at up to 720p.", Some(16), Some((1280, 720)), 23, Medium, 96),
        platform("email", "Email", "email", "Fits a 25 MB attachment at up to 720p.", Some(25), Some((1280, 720)), 23, Medium, 96),
    ]
}

pub fn catalog() -> PresetCatalog {
    PresetCatalog {
        quality: quality_presets(),
        platforms: platform_presets(),
        default_quality_id: DEFAULT_QUALITY_ID.into(),
    }
}

impl PresetCatalog {
    /// Like `PresetLibrary.GetById`: unknown ids fall back to the default preset.
    pub fn quality_or_default(&self, id: &str) -> Option<&QualityPreset> {
        self.quality
            .iter()
            .find(|p| p.id == id)
            .or_else(|| self.quality.iter().find(|p| p.id == self.default_quality_id))
            .or_else(|| self.quality.first())
    }

    pub fn platform(&self, id: &str) -> Option<&PlatformPreset> {
        self.platforms.iter().find(|p| p.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn quality_ids_in_csharp_order() {
        let ids: Vec<_> = quality_presets().into_iter().map(|p| p.id).collect();
        assert_eq!(ids, ["web-high", "web-medium", "web-low", "high-fast", "high-slow", "medium-fast", "medium-slow", "low-fast", "low-slow"]);
    }

    #[test]
    fn default_is_web_medium() {
        let c = catalog();
        assert_eq!(c.default_quality_id, "web-medium");
        let p = c.quality_or_default("web-medium").unwrap();
        assert_eq!(p.spec.crf, 23);
        assert_eq!(p.spec.codec, VideoCodec::H264);
        assert_eq!(p.spec.speed, SpeedPreset::Medium);
        assert_eq!(p.spec.audio_bitrate_kbps, 128);
        assert_eq!(p.name, "Medium h264 (Compatibility)");
        assert!(p.warning.is_none());
    }

    #[test]
    fn unknown_quality_falls_back_to_default() {
        assert_eq!(catalog().quality_or_default("nope").unwrap().id, "web-medium");
    }

    #[test]
    fn spot_check_specs_and_warnings() {
        let c = catalog();
        let low_slow = c.quality_or_default("low-slow").unwrap();
        assert_eq!(low_slow.spec.codec, VideoCodec::Av1);
        assert_eq!(low_slow.spec.crf, 36);
        assert_eq!(low_slow.spec.speed, SpeedPreset::Faster);
        assert_eq!(low_slow.spec.audio_codec, AudioCodec::Opus);
        assert_eq!(low_slow.warning.as_deref(), Some(AV1_WARNING));
        let high_fast = c.quality_or_default("high-fast").unwrap();
        assert_eq!(high_fast.category, PresetCategory::Optimization);
        assert_eq!(high_fast.warning.as_deref(), Some(H265_WARNING));
        assert_eq!(c.quality_or_default("web-high").unwrap().spec.audio_bitrate_kbps, 192);
    }

    #[test]
    fn platforms_match_table() {
        let c = catalog();
        let ids: Vec<_> = c.platforms.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, ["discord", "discord-nitro-basic", "discord-nitro", "youtube", "x", "vertical", "whatsapp", "email"]);
        let d = c.platform("discord").unwrap();
        assert_eq!(d.max_size_bytes, Some(10_000_000));
        assert_eq!((d.max_width, d.max_height), (Some(1920), Some(1080)));
        assert_eq!(d.spec.rate_control, RateControl::TargetSize);
        assert_eq!(d.spec.target_size_bytes, Some(10_000_000));
        assert_eq!(d.spec.target_bitrate_bps, None);
        assert_eq!(d.spec.speed, SpeedPreset::Medium);
        assert_eq!(d.spec.pixel_format, PixelFormat::Yuv420p);
        let y = c.platform("youtube").unwrap();
        assert_eq!(y.max_size_bytes, None);
        assert_eq!(y.spec.rate_control, RateControl::Crf);
        assert_eq!((y.spec.crf, y.spec.speed, y.spec.audio_bitrate_kbps), (18, SpeedPreset::Slow, 320));
        let v = c.platform("vertical").unwrap();
        assert_eq!((v.max_width, v.max_height, v.spec.crf), (Some(1080), Some(1920), 21));
        assert_eq!(c.platform("discord-nitro").unwrap().spec.audio_bitrate_kbps, 160);
        assert_eq!(c.platform("whatsapp").unwrap().max_size_bytes, Some(16_000_000));
        assert_eq!(c.platform("x").unwrap().max_height, Some(1200));
        for p in &c.platforms {
            assert_eq!(p.spec.codec, VideoCodec::H264);
            assert_eq!(p.spec.audio_codec, AudioCodec::Aac);
        }
    }

    #[test]
    fn ids_unique() {
        let c = catalog();
        let q: HashSet<_> = c.quality.iter().map(|p| &p.id).collect();
        let p: HashSet<_> = c.platforms.iter().map(|p| &p.id).collect();
        assert_eq!(q.len(), c.quality.len());
        assert_eq!(p.len(), c.platforms.len());
    }

    #[test]
    fn catalog_serializes_camel_case() {
        let json = serde_json::to_string(&catalog()).unwrap();
        assert!(json.contains("\"defaultQualityId\":\"web-medium\""));
        assert!(json.contains("\"maxSizeBytes\":10000000"));
        assert!(json.contains("\"rateControl\":\"targetSize\""));
    }
}
