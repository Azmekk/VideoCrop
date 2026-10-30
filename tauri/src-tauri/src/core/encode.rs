//! ffmpeg argument construction for the final encode. Port of EncodeCommandBuilder.cs.
//! Filter chain order is fixed: `crop=W:H:X:Y,scale=W2:H2:flags=lanczos`.

use super::models::*;

// ---------------------------------------------------------------- model helpers (Codecs.cs etc.)

impl VideoCodec {
    pub fn ffmpeg_encoder(self) -> &'static str {
        match self {
            VideoCodec::H264 => "libx264",
            VideoCodec::H265 => "libx265",
            VideoCodec::Av1 => "libsvtav1",
            VideoCodec::Vp9 => "libvpx-vp9",
        }
    }

    /// Default container extension, without the dot.
    pub fn default_container(self) -> &'static str {
        match self {
            VideoCodec::H264 | VideoCodec::H265 | VideoCodec::Av1 => "mp4",
            VideoCodec::Vp9 => "webm",
        }
    }

    /// Inclusive CRF range accepted by the encoder.
    pub fn crf_range(self) -> (u32, u32) {
        match self {
            VideoCodec::H264 | VideoCodec::H265 => (0, 51),
            VideoCodec::Av1 | VideoCodec::Vp9 => (0, 63),
        }
    }

    /// Human-readable name used in the pipeline summary.
    pub fn display_name(self) -> &'static str {
        match self {
            VideoCodec::H264 => "H.264",
            VideoCodec::H265 => "H.265",
            VideoCodec::Av1 => "AV1",
            VideoCodec::Vp9 => "VP9",
        }
    }
}

impl AudioCodec {
    pub fn ffmpeg_encoder(self) -> &'static str {
        match self {
            AudioCodec::Aac => "aac",
            AudioCodec::Opus => "libopus",
            AudioCodec::Mp3 => "libmp3lame",
            AudioCodec::Copy => "copy",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            AudioCodec::Aac => "AAC",
            AudioCodec::Opus => "Opus",
            AudioCodec::Mp3 => "MP3",
            AudioCodec::Copy => "copy",
        }
    }
}

impl SpeedPreset {
    /// x264/x265 preset names; AV1 (SVT-AV1) uses numeric presets.
    pub fn ffmpeg_preset(self, codec: VideoCodec) -> &'static str {
        if codec == VideoCodec::Av1 {
            return match self {
                SpeedPreset::Ultrafast => "12",
                SpeedPreset::Superfast => "11",
                SpeedPreset::Veryfast => "10",
                SpeedPreset::Faster => "9",
                SpeedPreset::Fast => "8",
                SpeedPreset::Medium => "6",
                SpeedPreset::Slow => "5",
                SpeedPreset::Slower => "4",
                SpeedPreset::Veryslow => "3",
            };
        }
        match self {
            SpeedPreset::Ultrafast => "ultrafast",
            SpeedPreset::Superfast => "superfast",
            SpeedPreset::Veryfast => "veryfast",
            SpeedPreset::Faster => "faster",
            SpeedPreset::Fast => "fast",
            SpeedPreset::Medium => "medium",
            SpeedPreset::Slow => "slow",
            SpeedPreset::Slower => "slower",
            SpeedPreset::Veryslow => "veryslow",
        }
    }
}

impl PixelFormat {
    pub fn ffmpeg_name(self) -> &'static str {
        match self {
            PixelFormat::Yuv420p => "yuv420p",
            PixelFormat::Yuv420p10le => "yuv420p10le",
        }
    }
}

impl CropSpec {
    /// Port of `CropSpec.Clamped`: keep inside the source, even size and even offset, min 2×2.
    pub fn clamped(&self, source_width: u32, source_height: u32) -> CropSpec {
        let sw = source_width as i64;
        let sh = source_height as i64;
        let mut x = (self.x as i64).clamp(0, (sw - 1).max(0));
        let mut y = (self.y as i64).clamp(0, (sh - 1).max(0));
        let max_w = sw - x;
        let max_h = sh - y;
        let mut w = (self.width as i64).clamp(1, max_w.max(1));
        let mut h = (self.height as i64).clamp(1, max_h.max(1));
        if w & 1 == 1 {
            w -= 1;
        }
        if h & 1 == 1 {
            h -= 1;
        }
        if x & 1 == 1 {
            x += 1;
        }
        if y & 1 == 1 {
            y += 1;
        }
        CropSpec { x: x as u32, y: y as u32, width: w.max(2) as u32, height: h.max(2) as u32 }
    }

    pub fn matches_source(&self, source_width: u32, source_height: u32) -> bool {
        self.x == 0 && self.y == 0 && self.width == source_width && self.height == source_height
    }
}

/// Port of `ResizeSpec.RoundEven`: rounds odd values down.
pub fn round_even(value: u32) -> u32 {
    value - (value & 1)
}

impl ResizeSpec {
    pub fn with_even_dimensions(&self) -> ResizeSpec {
        ResizeSpec { width: round_even(self.width), height: round_even(self.height), aspect_locked: self.aspect_locked }
    }
}

/// Height ffmpeg derives for `scale=W:-2` from an `in_w`×`in_h` input
/// (`av_rescale` rounds to nearest, then multiplies by 2).
pub fn derived_even_height(width: u32, in_w: u32, in_h: u32) -> u32 {
    if in_w == 0 {
        return round_even(width.max(2));
    }
    let num = width as u64 * in_h as u64;
    let den = in_w as u64 * 2;
    let half = (num + den / 2) / den;
    ((half * 2) as u32).max(2)
}

/// Finite and strictly greater than zero (false for NaN).
pub fn is_positive(x: f64) -> bool {
    x.is_finite() && x > 0.0
}

/// Seconds formatted like C# `"0.###"` (invariant, trailing zeros trimmed).
pub fn format_seconds(sec: f64) -> String {
    let sec = if sec.is_finite() { sec } else { 0.0 };
    let mut s = format!("{:.3}", sec);
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        s = "0".into();
    }
    s
}

// ---------------------------------------------------------------- builder

fn push(args: &mut Vec<String>, items: &[&str]) {
    args.extend(items.iter().map(|s| s.to_string()));
}

fn add_cut_before_input(args: &mut Vec<String>, cut: Option<&CutSpec>) {
    if let Some(c) = cut {
        if !c.accurate {
            push(args, &["-ss", &format_seconds(c.start_sec), "-to", &format_seconds(c.end_sec)]);
        }
    }
}

fn add_cut_after_input(args: &mut Vec<String>, cut: Option<&CutSpec>) {
    if let Some(c) = cut {
        if c.accurate {
            push(args, &["-ss", &format_seconds(c.start_sec), "-to", &format_seconds(c.end_sec)]);
        }
    }
}

fn add_video_encoder_args(args: &mut Vec<String>, c: &CompressionSpec, two_pass_bitrate: Option<u64>) -> Result<(), String> {
    push(args, &["-c:v", c.codec.ffmpeg_encoder()]);
    if c.codec != VideoCodec::Vp9 {
        push(args, &["-preset", c.speed.ffmpeg_preset(c.codec)]);
    }

    if let Some(bps) = two_pass_bitrate {
        push(args, &["-b:v", &bps.to_string()]);
    } else {
        match c.rate_control {
            RateControl::Crf => {
                push(args, &["-crf", &c.crf.to_string()]);
                if matches!(c.codec, VideoCodec::Av1 | VideoCodec::Vp9) {
                    push(args, &["-b:v", "0"]);
                }
            }
            RateControl::Cbr | RateControl::Vbr => {
                if let Some(bps) = c.target_bitrate_bps {
                    push(args, &["-b:v", &bps.to_string()]);
                    if c.rate_control == RateControl::Cbr {
                        push(args, &["-maxrate", &bps.to_string(), "-minrate", &bps.to_string(), "-bufsize", &(bps * 2).to_string()]);
                    }
                }
            }
            RateControl::TargetSize => {
                return Err("Target-size encoding must be run as two passes.".into());
            }
        }
    }

    push(args, &["-pix_fmt", c.pixel_format.ffmpeg_name()]);
    Ok(())
}

fn add_audio_args(args: &mut Vec<String>, c: &CompressionSpec) {
    push(args, &["-c:a", c.audio_codec.ffmpeg_encoder()]);
    if c.audio_codec != AudioCodec::Copy {
        push(args, &["-b:a", &format!("{}k", c.audio_bitrate_kbps)]);
    }
}

fn null_sink() -> &'static str {
    if cfg!(windows) {
        "NUL"
    } else {
        "/dev/null"
    }
}

/// Builds every ffmpeg invocation needed for `plan` (1 for single-pass, 2 for target-size).
/// `stats_prefix` is the `-passlogfile` prefix used only by two-pass.
/// Every invocation ends with `-progress pipe:1 -nostats <output>` (pass 1 writes to NUL).
///
/// Two-pass is used when the plan has a spec with `RateControl::TargetSize` and the mode
/// keeps video (audio-only target-size degrades to a single audio encode).
pub fn build_passes(plan: &JobPlan, input_path: &str, stats_prefix: &str) -> Result<Vec<Vec<String>>, String> {
    let want_video = plan.mode != OutputMode::AudioOnly;
    let two_pass = matches!(&plan.spec, Some(s) if s.rate_control == RateControl::TargetSize) && want_video;
    if two_pass {
        let (p1, p2) = build_two_pass(plan, input_path, stats_prefix)?;
        Ok(vec![p1, p2])
    } else {
        Ok(vec![build_single(plan, input_path)?])
    }
}

fn build_single(plan: &JobPlan, input_path: &str) -> Result<Vec<String>, String> {
    let cut = plan.cut.as_ref();
    let mut args: Vec<String> = Vec::new();
    push(&mut args, &["-y", "-hide_banner", "-nostats"]);
    add_cut_before_input(&mut args, cut);
    push(&mut args, &["-i", input_path]);
    add_cut_after_input(&mut args, cut);

    let want_video = plan.mode != OutputMode::AudioOnly;
    let want_audio = plan.mode != OutputMode::VideoOnly;

    match &plan.spec {
        None => {
            if want_video {
                push(&mut args, &["-c:v", "copy"]);
            } else {
                args.push("-vn".into());
            }
            if want_audio {
                push(&mut args, &["-c:a", "copy"]);
            } else {
                args.push("-an".into());
            }
        }
        Some(spec) => {
            if want_video {
                if let Some(f) = video_filter(plan.crop.as_ref(), plan.resize.as_ref()) {
                    push(&mut args, &["-vf", &f]);
                }
                add_video_encoder_args(&mut args, spec, None)?;
            } else {
                args.push("-vn".into());
            }
            if want_audio {
                add_audio_args(&mut args, spec);
            } else {
                args.push("-an".into());
            }
        }
    }

    if want_video {
        push(&mut args, &["-map", "0:v:0"]);
    }
    if want_audio {
        push(&mut args, &["-map", "0:a:0?"]);
    }
    push(&mut args, &["-map_metadata", "-1", "-progress", "pipe:1", "-nostats", &plan.output_path]);
    Ok(args)
}

fn build_two_pass(plan: &JobPlan, input_path: &str, stats_prefix: &str) -> Result<(Vec<String>, Vec<String>), String> {
    let comp = plan.spec.as_ref().ok_or("Two-pass requires a compression spec.")?;
    let target = comp
        .target_bitrate_bps
        .ok_or("Two-pass requires a target bitrate (plan the job first).")?;
    let cut = plan.cut.as_ref();
    let filter = video_filter(plan.crop.as_ref(), plan.resize.as_ref());
    let want_audio = plan.mode != OutputMode::VideoOnly;

    let mut p1: Vec<String> = Vec::new();
    push(&mut p1, &["-y", "-hide_banner", "-nostats"]);
    add_cut_before_input(&mut p1, cut);
    push(&mut p1, &["-i", input_path]);
    add_cut_after_input(&mut p1, cut);
    if let Some(f) = &filter {
        push(&mut p1, &["-vf", f]);
    }
    add_video_encoder_args(&mut p1, comp, Some(target))?;
    push(&mut p1, &["-pass", "1", "-passlogfile", stats_prefix, "-an", "-f", "mp4", "-map", "0:v:0", null_sink()]);

    let mut p2: Vec<String> = Vec::new();
    push(&mut p2, &["-y", "-hide_banner", "-nostats"]);
    add_cut_before_input(&mut p2, cut);
    push(&mut p2, &["-i", input_path]);
    add_cut_after_input(&mut p2, cut);
    if let Some(f) = &filter {
        push(&mut p2, &["-vf", f]);
    }
    add_video_encoder_args(&mut p2, comp, Some(target))?;
    push(&mut p2, &["-pass", "2", "-passlogfile", stats_prefix]);
    if want_audio {
        add_audio_args(&mut p2, comp);
    } else {
        p2.push("-an".into());
    }
    push(&mut p2, &["-map", "0:v:0"]);
    if want_audio {
        push(&mut p2, &["-map", "0:a:0?"]);
    }
    push(&mut p2, &["-map_metadata", "-1", "-progress", "pipe:1", "-nostats", &plan.output_path]);

    Ok((p1, p2))
}

/// `crop=...,scale=...` or None when neither is set.
pub fn video_filter(crop: Option<&CropSpec>, resize: Option<&ResizeSpec>) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if let Some(c) = crop {
        parts.push(format!("crop={}:{}:{}:{}", c.width, c.height, c.x, c.y));
    }
    if let Some(r) = resize {
        let r = r.with_even_dimensions();
        if r.aspect_locked {
            parts.push(format!("scale={}:-2:flags=lanczos", r.width));
        } else {
            parts.push(format!("scale={}:{}:flags=lanczos", r.width, r.height));
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(","))
    }
}

/// Floor for computed video bitrates (matches C#).
pub const MIN_VIDEO_BITRATE_BPS: u64 = 50_000;

/// Video bitrate that makes `target_bytes` after reserving audio, with a safety margin (0.97).
/// A non-positive duration returns the floor (the planner rejects such jobs earlier).
pub fn compute_target_bitrate(target_bytes: u64, duration_sec: f64, audio_kbps: u32, safety_margin: f64) -> u64 {
    if !is_positive(duration_sec) {
        return MIN_VIDEO_BITRATE_BPS;
    }
    let audio_bps = audio_kbps as i128 * 1000;
    let total_bits = (target_bytes as f64 * 8.0 * safety_margin) as i128;
    let audio_bits = (audio_bps as f64 * duration_sec) as i128;
    let video_bits = total_bits - audio_bits;
    let video_bps = (video_bits as f64 / duration_sec) as i128;
    video_bps.max(MIN_VIDEO_BITRATE_BPS as i128).min(u64::MAX as i128) as u64
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    const IN: &str = r"C:\in\movie.mp4";
    const OUT: &str = r"C:\out\movie_VideoCrop.mp4";

    fn web_medium() -> CompressionSpec {
        CompressionSpec {
            codec: VideoCodec::H264,
            rate_control: RateControl::Crf,
            crf: 23,
            target_bitrate_bps: None,
            target_size_bytes: None,
            speed: SpeedPreset::Medium,
            audio_codec: AudioCodec::Aac,
            audio_bitrate_kbps: 128,
            pixel_format: PixelFormat::Yuv420p,
            container: None,
        }
    }

    fn spec(codec: VideoCodec, rc: RateControl) -> CompressionSpec {
        CompressionSpec { codec, rate_control: rc, ..web_medium() }
    }

    fn plan_with(cut: Option<CutSpec>, crop: Option<CropSpec>, resize: Option<ResizeSpec>, comp: Option<CompressionSpec>, mode: OutputMode) -> JobPlan {
        JobPlan {
            output_path: OUT.into(),
            summary: String::new(),
            output_width: None,
            output_height: None,
            two_pass: matches!(&comp, Some(c) if c.rate_control == RateControl::TargetSize),
            spec: comp,
            crop,
            resize,
            cut,
            mode,
            estimated_size_bytes: None,
            warnings: vec![],
            size_alert: None,
        }
    }

    fn base(cut: Option<CutSpec>, crop: Option<CropSpec>, resize: Option<ResizeSpec>) -> JobPlan {
        plan_with(cut, crop, resize, Some(web_medium()), OutputMode::Everything)
    }

    fn single(plan: &JobPlan) -> Vec<String> {
        let mut passes = build_passes(plan, IN, "stats").unwrap();
        assert_eq!(passes.len(), 1);
        passes.remove(0)
    }

    fn idx(args: &[String], s: &str) -> Option<usize> {
        args.iter().position(|a| a == s)
    }

    fn has(args: &[String], s: &str) -> bool {
        idx(args, s).is_some()
    }

    fn value_after(args: &[String], flag: &str) -> Option<String> {
        idx(args, flag).and_then(|i| args.get(i + 1).cloned())
    }

    fn crop(x: u32, y: u32, w: u32, h: u32) -> CropSpec {
        CropSpec { x, y, width: w, height: h }
    }

    fn rs(w: u32, h: u32, locked: bool) -> ResizeSpec {
        ResizeSpec { width: w, height: h, aspect_locked: locked }
    }

    fn cut(a: f64, b: f64, accurate: bool) -> CutSpec {
        CutSpec { start_sec: a, end_sec: b, accurate }
    }

    // ---- ported from EncodeCommandBuilderTests.cs

    #[test]
    fn base_encode_includes_codec_and_audio() {
        let a = single(&base(None, None, None));
        for s in ["-i", IN, "-c:v", "libx264", "-crf", "23", "-c:a", "aac", "-b:a", "128k", "-pix_fmt", "yuv420p", "-progress", "pipe:1"] {
            assert!(has(&a, s), "missing {s} in {a:?}");
        }
    }

    #[test]
    fn crop_only_produces_crop_filter() {
        let c = crop(10, 20, 1600, 900);
        assert_eq!(video_filter(Some(&c), None).as_deref(), Some("crop=1600:900:10:20"));
        let a = single(&base(None, Some(c), None));
        assert_eq!(value_after(&a, "-vf").as_deref(), Some("crop=1600:900:10:20"));
    }

    #[test]
    fn resize_only_locked_uses_lanczos() {
        assert_eq!(video_filter(None, Some(&rs(1280, 720, true))).as_deref(), Some("scale=1280:-2:flags=lanczos"));
    }

    #[test]
    fn resize_unlocked_uses_explicit_dimensions() {
        assert_eq!(video_filter(None, Some(&rs(1280, 800, false))).as_deref(), Some("scale=1280:800:flags=lanczos"));
    }

    #[test]
    fn crop_and_resize_are_ordered_crop_then_scale() {
        assert_eq!(
            video_filter(Some(&crop(0, 0, 1600, 900)), Some(&rs(1280, 720, true))).as_deref(),
            Some("crop=1600:900:0:0,scale=1280:-2:flags=lanczos")
        );
    }

    #[test]
    fn neither_crop_nor_resize_yields_no_filter() {
        assert_eq!(video_filter(None, None), None);
        let a = single(&base(None, None, None));
        assert!(!has(&a, "-vf"));
    }

    #[test]
    fn resize_rounds_odd_dimensions_to_even() {
        assert_eq!(video_filter(None, Some(&rs(1281, 723, false))).as_deref(), Some("scale=1280:722:flags=lanczos"));
    }

    #[test]
    fn resize_locked_rounds_odd_width() {
        assert_eq!(video_filter(None, Some(&rs(1281, 0, true))).as_deref(), Some("scale=1280:-2:flags=lanczos"));
    }

    #[test]
    fn cut_inaccurate_emits_ss_before_input() {
        let a = single(&base(Some(cut(2.0, 7.0, false)), None, None));
        let ss = idx(&a, "-ss").unwrap();
        let i = idx(&a, "-i").unwrap();
        assert!(i > ss);
        assert_eq!(&a[..9], &["-y", "-hide_banner", "-nostats", "-ss", "2", "-to", "7", "-i", IN]);
    }

    #[test]
    fn cut_accurate_emits_ss_after_input() {
        let a = single(&base(Some(cut(2.0, 7.0, true)), None, None));
        let ss = idx(&a, "-ss").unwrap();
        let i = idx(&a, "-i").unwrap();
        assert!(ss > i);
        assert_eq!(&a[..9], &["-y", "-hide_banner", "-nostats", "-i", IN, "-ss", "2", "-to", "7"]);
    }

    #[test]
    fn cut_fractional_seconds_formatting() {
        let a = single(&base(Some(cut(1.5, 12.3456, false)), None, None));
        assert_eq!(value_after(&a, "-ss").as_deref(), Some("1.5"));
        assert_eq!(value_after(&a, "-to").as_deref(), Some("12.346"));
    }

    #[test]
    fn av1_uses_libsvtav1_with_preset_number() {
        let s = CompressionSpec { crf: 28, ..spec(VideoCodec::Av1, RateControl::Crf) };
        let a = single(&plan_with(None, None, None, Some(s), OutputMode::Everything));
        assert_eq!(value_after(&a, "-c:v").as_deref(), Some("libsvtav1"));
        assert_eq!(value_after(&a, "-preset").as_deref(), Some("6"));
        assert_eq!(value_after(&a, "-crf").as_deref(), Some("28"));
        assert_eq!(value_after(&a, "-b:v").as_deref(), Some("0"));
    }

    #[test]
    fn two_pass_emits_pass1_and_pass2_with_stats_prefix() {
        let s = CompressionSpec { target_bitrate_bps: Some(2_000_000), ..spec(VideoCodec::H264, RateControl::TargetSize) };
        let p = plan_with(None, None, None, Some(s), OutputMode::Everything);
        let passes = build_passes(&p, IN, r"C:\tmp\videocrop_pass").unwrap();
        assert_eq!(passes.len(), 2);
        assert_eq!(value_after(&passes[0], "-pass").as_deref(), Some("1"));
        assert!(has(&passes[0], "-an"));
        assert_eq!(value_after(&passes[0], "-passlogfile").as_deref(), Some(r"C:\tmp\videocrop_pass"));
        assert_eq!(value_after(&passes[1], "-pass").as_deref(), Some("2"));
        assert_eq!(value_after(&passes[1], "-passlogfile").as_deref(), Some(r"C:\tmp\videocrop_pass"));
    }

    #[test]
    fn compute_target_bitrate_accounts_for_audio() {
        let bps = compute_target_bitrate(8 * 1024 * 1024, 10.0, 128, 0.97);
        assert!(bps > 5_000_000 && bps < 7_000_000, "{bps}");
    }

    // ---- exact argument order

    #[test]
    fn single_pass_full_argument_order() {
        let a = single(&base(None, Some(crop(0, 0, 1600, 900)), Some(rs(1280, 720, true))));
        let expected = [
            "-y", "-hide_banner", "-nostats", "-i", IN,
            "-vf", "crop=1600:900:0:0,scale=1280:-2:flags=lanczos",
            "-c:v", "libx264", "-preset", "medium", "-crf", "23", "-pix_fmt", "yuv420p",
            "-c:a", "aac", "-b:a", "128k",
            "-map", "0:v:0", "-map", "0:a:0?", "-map_metadata", "-1",
            "-progress", "pipe:1", "-nostats", OUT,
        ];
        assert_eq!(a, expected);
    }

    #[test]
    fn two_pass_full_argument_order() {
        let s = CompressionSpec { target_bitrate_bps: Some(1_234_567), ..spec(VideoCodec::H264, RateControl::TargetSize) };
        let p = plan_with(Some(cut(1.0, 4.25, false)), Some(crop(2, 4, 100, 50)), None, Some(s), OutputMode::Everything);
        let passes = build_passes(&p, IN, "pfx").unwrap();
        let sink = if cfg!(windows) { "NUL" } else { "/dev/null" };
        let p1 = [
            "-y", "-hide_banner", "-nostats", "-ss", "1", "-to", "4.25", "-i", IN,
            "-vf", "crop=100:50:2:4",
            "-c:v", "libx264", "-preset", "medium", "-b:v", "1234567", "-pix_fmt", "yuv420p",
            "-pass", "1", "-passlogfile", "pfx", "-an", "-f", "mp4", "-map", "0:v:0", sink,
        ];
        let p2 = [
            "-y", "-hide_banner", "-nostats", "-ss", "1", "-to", "4.25", "-i", IN,
            "-vf", "crop=100:50:2:4",
            "-c:v", "libx264", "-preset", "medium", "-b:v", "1234567", "-pix_fmt", "yuv420p",
            "-pass", "2", "-passlogfile", "pfx",
            "-c:a", "aac", "-b:a", "128k",
            "-map", "0:v:0", "-map", "0:a:0?", "-map_metadata", "-1",
            "-progress", "pipe:1", "-nostats", OUT,
        ];
        assert_eq!(passes[0], p1);
        assert_eq!(passes[1], p2);
    }

    #[test]
    fn two_pass_accurate_cut_after_input_in_both_passes() {
        let s = CompressionSpec { target_bitrate_bps: Some(1_000_000), ..spec(VideoCodec::H264, RateControl::TargetSize) };
        let p = plan_with(Some(cut(3.0, 9.0, true)), None, None, Some(s), OutputMode::Everything);
        for a in build_passes(&p, IN, "pfx").unwrap() {
            assert!(idx(&a, "-ss").unwrap() > idx(&a, "-i").unwrap());
        }
    }

    #[test]
    fn two_pass_supported_for_every_codec() {
        for (codec, enc) in [
            (VideoCodec::H264, "libx264"),
            (VideoCodec::H265, "libx265"),
            (VideoCodec::Vp9, "libvpx-vp9"),
            (VideoCodec::Av1, "libsvtav1"),
        ] {
            let s = CompressionSpec { target_bitrate_bps: Some(900_000), ..spec(codec, RateControl::TargetSize) };
            let p = plan_with(None, None, None, Some(s), OutputMode::Everything);
            let passes = build_passes(&p, IN, "pfx").unwrap();
            assert_eq!(passes.len(), 2, "{codec:?}");
            for a in &passes {
                assert_eq!(value_after(a, "-c:v").as_deref(), Some(enc));
                assert_eq!(value_after(a, "-b:v").as_deref(), Some("900000"));
                assert!(!has(a, "-crf"));
            }
            assert_eq!(has(&passes[0], "-preset"), codec != VideoCodec::Vp9);
            if codec == VideoCodec::Av1 {
                assert_eq!(value_after(&passes[0], "-preset").as_deref(), Some("6"));
            }
        }
    }

    #[test]
    fn two_pass_without_bitrate_errors() {
        let p = plan_with(None, None, None, Some(spec(VideoCodec::H264, RateControl::TargetSize)), OutputMode::Everything);
        assert!(build_passes(&p, IN, "pfx").is_err());
    }

    #[test]
    fn two_pass_video_only_drops_audio() {
        let s = CompressionSpec { target_bitrate_bps: Some(1_000_000), ..spec(VideoCodec::H264, RateControl::TargetSize) };
        let p = plan_with(None, None, None, Some(s), OutputMode::VideoOnly);
        let passes = build_passes(&p, IN, "pfx").unwrap();
        assert!(has(&passes[1], "-an"));
        assert!(!has(&passes[1], "-c:a"));
        assert!(!has(&passes[1], "0:a:0?"));
    }

    #[test]
    fn target_size_audio_only_is_single_audio_pass() {
        let s = CompressionSpec { target_bitrate_bps: Some(1_000_000), ..spec(VideoCodec::H264, RateControl::TargetSize) };
        let p = plan_with(None, None, None, Some(s), OutputMode::AudioOnly);
        let a = single(&p);
        assert!(has(&a, "-vn"));
        assert!(!has(&a, "-c:v"));
        assert!(!has(&a, "-pass"));
        assert_eq!(value_after(&a, "-c:a").as_deref(), Some("aac"));
    }

    #[test]
    fn stream_copy_everything() {
        let p = plan_with(Some(cut(1.0, 2.0, false)), None, None, None, OutputMode::Everything);
        let a = single(&p);
        assert_eq!(
            a,
            [
                "-y", "-hide_banner", "-nostats", "-ss", "1", "-to", "2", "-i", IN,
                "-c:v", "copy", "-c:a", "copy",
                "-map", "0:v:0", "-map", "0:a:0?", "-map_metadata", "-1",
                "-progress", "pipe:1", "-nostats", OUT,
            ]
        );
    }

    #[test]
    fn stream_copy_ignores_crop_and_resize() {
        let p = plan_with(None, Some(crop(0, 0, 100, 100)), Some(rs(64, 64, false)), None, OutputMode::Everything);
        assert!(!has(&single(&p), "-vf"));
    }

    #[test]
    fn stream_copy_audio_only() {
        let a = single(&plan_with(None, None, None, None, OutputMode::AudioOnly));
        assert!(has(&a, "-vn"));
        assert_eq!(value_after(&a, "-c:a").as_deref(), Some("copy"));
        assert!(!has(&a, "0:v:0"));
        assert!(has(&a, "0:a:0?"));
    }

    #[test]
    fn stream_copy_video_only() {
        let a = single(&plan_with(None, None, None, None, OutputMode::VideoOnly));
        assert!(has(&a, "-an"));
        assert_eq!(value_after(&a, "-c:v").as_deref(), Some("copy"));
        assert!(has(&a, "0:v:0"));
        assert!(!has(&a, "0:a:0?"));
    }

    #[test]
    fn audio_only_reencode_has_no_video_args_or_filter() {
        let p = plan_with(None, Some(crop(0, 0, 100, 100)), Some(rs(64, 64, true)), Some(web_medium()), OutputMode::AudioOnly);
        let a = single(&p);
        assert!(has(&a, "-vn"));
        for s in ["-vf", "-c:v", "-pix_fmt", "-crf", "0:v:0"] {
            assert!(!has(&a, s), "{s}");
        }
        assert_eq!(value_after(&a, "-b:a").as_deref(), Some("128k"));
    }

    #[test]
    fn video_only_reencode_drops_audio() {
        let a = single(&plan_with(None, None, None, Some(web_medium()), OutputMode::VideoOnly));
        assert!(has(&a, "-an"));
        assert!(!has(&a, "-c:a"));
        assert!(has(&a, "libx264"));
    }

    #[test]
    fn audio_copy_has_no_bitrate() {
        let s = CompressionSpec { audio_codec: AudioCodec::Copy, ..web_medium() };
        let a = single(&plan_with(None, None, None, Some(s), OutputMode::Everything));
        assert_eq!(value_after(&a, "-c:a").as_deref(), Some("copy"));
        assert!(!has(&a, "-b:a"));
    }

    #[test]
    fn audio_encoder_names() {
        for (codec, name) in [(AudioCodec::Aac, "aac"), (AudioCodec::Opus, "libopus"), (AudioCodec::Mp3, "libmp3lame")] {
            let s = CompressionSpec { audio_codec: codec, audio_bitrate_kbps: 96, ..web_medium() };
            let a = single(&plan_with(None, None, None, Some(s), OutputMode::Everything));
            assert_eq!(value_after(&a, "-c:a").as_deref(), Some(name));
            assert_eq!(value_after(&a, "-b:a").as_deref(), Some("96k"));
        }
    }

    #[test]
    fn vp9_crf_has_no_preset_and_zero_bitrate() {
        let s = CompressionSpec { crf: 31, ..spec(VideoCodec::Vp9, RateControl::Crf) };
        let a = single(&plan_with(None, None, None, Some(s), OutputMode::Everything));
        assert_eq!(value_after(&a, "-c:v").as_deref(), Some("libvpx-vp9"));
        assert!(!has(&a, "-preset"));
        assert_eq!(value_after(&a, "-crf").as_deref(), Some("31"));
        assert_eq!(value_after(&a, "-b:v").as_deref(), Some("0"));
    }

    #[test]
    fn h265_crf_uses_preset_name_and_no_bv() {
        let s = CompressionSpec { speed: SpeedPreset::Slow, ..spec(VideoCodec::H265, RateControl::Crf) };
        let a = single(&plan_with(None, None, None, Some(s), OutputMode::Everything));
        assert_eq!(value_after(&a, "-c:v").as_deref(), Some("libx265"));
        assert_eq!(value_after(&a, "-preset").as_deref(), Some("slow"));
        assert!(!has(&a, "-b:v"));
    }

    #[test]
    fn cbr_sets_max_min_and_bufsize() {
        let s = CompressionSpec { target_bitrate_bps: Some(3_000_000), ..spec(VideoCodec::H264, RateControl::Cbr) };
        let a = single(&plan_with(None, None, None, Some(s), OutputMode::Everything));
        assert_eq!(value_after(&a, "-b:v").as_deref(), Some("3000000"));
        assert_eq!(value_after(&a, "-maxrate").as_deref(), Some("3000000"));
        assert_eq!(value_after(&a, "-minrate").as_deref(), Some("3000000"));
        assert_eq!(value_after(&a, "-bufsize").as_deref(), Some("6000000"));
        assert!(!has(&a, "-crf"));
    }

    #[test]
    fn vbr_sets_only_bitrate() {
        let s = CompressionSpec { target_bitrate_bps: Some(2_500_000), ..spec(VideoCodec::H264, RateControl::Vbr) };
        let a = single(&plan_with(None, None, None, Some(s), OutputMode::Everything));
        assert_eq!(value_after(&a, "-b:v").as_deref(), Some("2500000"));
        assert!(!has(&a, "-maxrate"));
    }

    #[test]
    fn ten_bit_pixel_format() {
        let s = CompressionSpec { pixel_format: PixelFormat::Yuv420p10le, ..web_medium() };
        let a = single(&plan_with(None, None, None, Some(s), OutputMode::Everything));
        assert_eq!(value_after(&a, "-pix_fmt").as_deref(), Some("yuv420p10le"));
    }

    #[test]
    fn av1_preset_mapping_is_numeric() {
        let expected = [
            (SpeedPreset::Ultrafast, "12"), (SpeedPreset::Superfast, "11"), (SpeedPreset::Veryfast, "10"),
            (SpeedPreset::Faster, "9"), (SpeedPreset::Fast, "8"), (SpeedPreset::Medium, "6"),
            (SpeedPreset::Slow, "5"), (SpeedPreset::Slower, "4"), (SpeedPreset::Veryslow, "3"),
        ];
        for (p, n) in expected {
            assert_eq!(p.ffmpeg_preset(VideoCodec::Av1), n);
        }
        assert_eq!(SpeedPreset::Veryslow.ffmpeg_preset(VideoCodec::H264), "veryslow");
    }

    #[test]
    fn container_mapping_and_crf_ranges() {
        assert_eq!(VideoCodec::H264.default_container(), "mp4");
        assert_eq!(VideoCodec::H265.default_container(), "mp4");
        assert_eq!(VideoCodec::Av1.default_container(), "mp4");
        assert_eq!(VideoCodec::Vp9.default_container(), "webm");
        assert_eq!(VideoCodec::H264.crf_range(), (0, 51));
        assert_eq!(VideoCodec::Av1.crf_range(), (0, 63));
    }

    // ---- bitrate math

    #[test]
    fn compute_target_bitrate_exact_values() {
        // 10 MB * 8 * 0.97 = 77_600_000 bits over 10 s = 7_760_000; minus 128k audio.
        assert_eq!(compute_target_bitrate(10_000_000, 10.0, 128, 0.97), 7_632_000);
        assert_eq!(compute_target_bitrate(10_000_000, 10.0, 0, 1.0), 8_000_000);
    }

    #[test]
    fn compute_target_bitrate_floors_at_50k() {
        assert_eq!(compute_target_bitrate(1_000_000, 3600.0, 320, 0.97), 50_000);
        assert_eq!(compute_target_bitrate(1_000_000, 0.0, 128, 0.97), 50_000);
        assert_eq!(compute_target_bitrate(1_000_000, -1.0, 128, 0.97), 50_000);
    }

    // ---- helpers

    #[test]
    fn format_seconds_matches_csharp() {
        assert_eq!(format_seconds(0.0), "0");
        assert_eq!(format_seconds(2.0), "2");
        assert_eq!(format_seconds(2.5), "2.5");
        assert_eq!(format_seconds(2.1234), "2.123");
        assert_eq!(format_seconds(59.9999), "60");
        assert_eq!(format_seconds(3600.25), "3600.25");
    }

    #[test]
    fn crop_clamped_rounds_to_even() {
        assert_eq!(crop(1, 3, 101, 51).clamped(1920, 1080), crop(2, 4, 100, 50));
        assert_eq!(crop(0, 0, 5000, 5000).clamped(1920, 1080), crop(0, 0, 1920, 1080));
        assert_eq!(crop(1900, 1070, 100, 100).clamped(1920, 1080), crop(1900, 1070, 20, 10));
        assert_eq!(crop(0, 0, 1, 1).clamped(1920, 1080), crop(0, 0, 2, 2));
        assert!(crop(0, 0, 1920, 1080).matches_source(1920, 1080));
        assert!(!crop(2, 0, 1918, 1080).matches_source(1920, 1080));
    }

    #[test]
    fn derived_even_height_matches_ffmpeg() {
        assert_eq!(derived_even_height(1280, 1920, 1080), 720);
        assert_eq!(derived_even_height(1280, 1600, 900), 720);
        assert_eq!(derived_even_height(640, 1000, 1000), 640);
        // 853 * 1080 / 1920 = 479.8 -> /2 = 239.9 -> 240*2 = 480
        assert_eq!(derived_even_height(853, 1920, 1080), 480);
        assert_eq!(derived_even_height(608, 1080, 1920), 1080);
    }
}
