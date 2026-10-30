//! Turns a `JobDraft` into a fully resolved `JobPlan`: output path, effective spec (platform
//! presets resolved for this source/cut), effective resize, final dimensions, size estimate,
//! warnings and the human-readable pipeline summary.

use std::path::Path;

use super::encode::{compute_target_bitrate, derived_even_height, is_positive, round_even};
use super::models::*;
use super::naming;
use super::presets::MB;

/// Minimum cut length (seconds).
pub const MIN_CUT_SEC: f64 = 0.1;
/// Safety margin used for every target-size bitrate.
pub const SAFETY_MARGIN: f64 = 0.97;
/// Resolution ladder (short side) tried by platform presets, largest first.
pub const RESOLUTION_LADDER: [u32; 7] = [2160, 1440, 1080, 720, 540, 480, 360];
/// Minimum bits per pixel per frame for acceptable H.264 quality.
pub const MIN_BITS_PER_PIXEL: f64 = 0.06;
/// Fallback rung when nothing on the ladder reaches `MIN_BITS_PER_PIXEL`.
pub const FALLBACK_SHORT_SIDE: u32 = 360;

/// Quality ceiling for target-size encodes, by output short side and frame rate. Stops a
/// short clip from ballooning to the full cap (a 10 s clip doesn't need 500 MB).
pub fn quality_ceiling_bps(short_side: u32, fps: f64) -> u64 {
    let base: u64 = match short_side {
        0..=480 => 2_500_000,
        481..=720 => 5_000_000,
        721..=1080 => 8_000_000,
        1081..=1440 => 12_000_000,
        _ => 20_000_000,
    };
    if fps > 30.5 {
        base * 3 / 2
    } else {
        base
    }
}

/// `10_000_000` -> "10 MB", `9_700_000` -> "9.7 MB", `485_000_000` -> "485 MB" (decimal MB).
pub fn format_mb(bytes: u64) -> String {
    let v = bytes as f64 / MB as f64;
    let s = if v >= 100.0 { format!("{:.0}", v) } else { format!("{:.1}", v) };
    let s = s.strip_suffix(".0").map(str::to_string).unwrap_or(s);
    format!("{s} MB")
}

enum Resolved<'a> {
    Copy,
    Encode { spec: CompressionSpec, platform: Option<&'a PlatformPreset> },
}

pub fn plan(info: &VideoInfo, draft: &JobDraft, catalog: &PresetCatalog, exists: impl Fn(&Path) -> bool) -> Result<JobPlan, String> {
    let mode = draft.mode;
    let has_source_audio = !info.audio_streams.is_empty();
    if mode == OutputMode::AudioOnly && !has_source_audio {
        return Err("The source has no audio track.".into());
    }

    // ---- cut
    let cut = normalize_cut(draft.cut, info.duration_sec)?;
    let duration = match &cut {
        Some(c) => c.end_sec - c.start_sec,
        None => info.duration_sec.max(0.0),
    };

    // ---- compression
    let resolved = resolve_compression(&draft.compression, catalog)?;
    let draft_had_filters = draft.crop.is_some() || draft.resize.is_some();
    let want_video = mode != OutputMode::AudioOnly;
    let want_audio = mode != OutputMode::VideoOnly;
    let fps = if info.fps.is_finite() && info.fps > 0.0 { info.fps } else { 30.0 };

    let mut warnings: Vec<String> = Vec::new();
    let mut crop: Option<CropSpec> = None;
    let mut resize: Option<ResizeSpec> = None;
    let mut spec: Option<CompressionSpec> = None;
    let mut two_pass = false;
    // Audio bitrate assumed for size math (0 when no audio goes into the output).
    let audio_kbps_for_size = |s: &CompressionSpec| -> u32 {
        if !want_audio || !has_source_audio {
            return 0;
        }
        if s.audio_codec == AudioCodec::Copy {
            if let Some(bps) = info.audio_streams[0].bitrate_bps {
                return bps.div_ceil(1000) as u32;
            }
        }
        s.audio_bitrate_kbps
    };

    if let Resolved::Encode { spec: base, platform } = resolved {
        let mut s = base;

        if want_video {
            // ---- crop (source pixels, clamped even)
            crop = draft
                .crop
                .map(|c| c.clamped(info.width, info.height))
                .filter(|c| !c.matches_source(info.width, info.height));
            let (pw, ph) = crop.map(|c| (c.width, c.height)).unwrap_or((info.width, info.height));

            // ---- user resize (post-crop frame)
            if let Some(r) = draft.resize {
                if r.width < 2 || (!r.aspect_locked && r.height < 2) {
                    return Err("Resize width and height must be at least 2 px.".into());
                }
                resize = Some(normalized_resize(r, pw, ph));
            }

            if let Some(p) = platform {
                let audio_kbps = audio_kbps_for_size(&s);
                let outcome = resolve_platform(p, &s, pw, ph, fps, duration, audio_kbps, resize)?;
                resize = outcome.resize;
                warnings.extend(outcome.warnings);
                if let Some(bps) = outcome.video_bps {
                    s.rate_control = RateControl::TargetSize;
                    s.target_size_bytes = p.max_size_bytes;
                    s.target_bitrate_bps = Some(bps);
                    two_pass = true;
                }
            } else if s.rate_control == RateControl::TargetSize {
                let target = s.target_size_bytes.filter(|b| *b > 0).ok_or("Set a target file size.")?;
                if !is_positive(duration) {
                    return Err("Can't target a file size: the clip duration is unknown.".into());
                }
                s.target_bitrate_bps = Some(compute_target_bitrate(target, duration, audio_kbps_for_size(&s), SAFETY_MARGIN));
                two_pass = true;
            }
        }
        spec = Some(s);
    }

    // ---- output path
    let ext = output_extension(info, spec.as_ref(), mode);
    let dir = match draft.output_dir.as_deref().map(str::trim).filter(|d| !d.is_empty()) {
        Some(d) => d.to_string(),
        None => Path::new(&info.path)
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| ".".to_string()),
    };
    let output_path = naming::next_available(&info.path, &dir, &ext, exists)?
        .to_string_lossy()
        .into_owned();

    // ---- final frame size
    let (output_width, output_height) = if !want_video {
        (None, None)
    } else if spec.is_none() {
        (Some(info.width), Some(info.height))
    } else {
        let (pw, ph) = crop.map(|c| (c.width, c.height)).unwrap_or((info.width, info.height));
        let (w, h) = match resize {
            Some(r) => resize_output(r, pw, ph),
            None => (pw, ph),
        };
        (Some(w), Some(h))
    };

    // ---- size estimate
    let estimated_size_bytes = estimate_size(info, spec.as_ref(), mode, duration, &audio_kbps_for_size);

    // ---- summary
    let summary = build_summary(info, spec.as_ref(), mode, cut.is_some(), draft_had_filters, crop, resize, (output_width, output_height), two_pass, estimated_size_bytes);

    Ok(JobPlan {
        output_path,
        summary,
        output_width,
        output_height,
        spec,
        crop,
        resize,
        cut,
        mode,
        two_pass,
        estimated_size_bytes,
        warnings,
    })
}

/// Clamps to the source, drops a full-range cut, enforces the 100 ms minimum.
fn normalize_cut(cut: Option<CutSpec>, source_duration: f64) -> Result<Option<CutSpec>, String> {
    let Some(c) = cut else { return Ok(None) };
    if !c.start_sec.is_finite() || !c.end_sec.is_finite() {
        return Err("Cut times are invalid.".into());
    }
    let known = source_duration.is_finite() && source_duration > 0.0;
    let start = c.start_sec.max(0.0);
    let end = if known { c.end_sec.min(source_duration) } else { c.end_sec };
    if end - start < MIN_CUT_SEC - 1e-9 {
        return Err("The cut must be at least 0.1 s long.".into());
    }
    if known && start <= 0.0 && end >= source_duration - 0.001 {
        return Ok(None);
    }
    Ok(Some(CutSpec { start_sec: start, end_sec: end, accurate: c.accurate }))
}

fn resolve_compression<'a>(choice: &CompressionChoice, catalog: &'a PresetCatalog) -> Result<Resolved<'a>, String> {
    Ok(match choice {
        CompressionChoice::Off => Resolved::Copy,
        CompressionChoice::Quality { preset_id } => {
            let p = catalog.quality_or_default(preset_id).ok_or("No quality presets available.")?;
            Resolved::Encode { spec: p.spec.clone(), platform: None }
        }
        CompressionChoice::Platform { platform_id } => {
            let p = catalog
                .platform(platform_id)
                .ok_or_else(|| format!("Unknown platform preset \"{platform_id}\"."))?;
            Resolved::Encode { spec: p.spec.clone(), platform: Some(p) }
        }
        CompressionChoice::Custom { spec } => {
            let mut s = spec.clone();
            let (lo, hi) = s.codec.crf_range();
            s.crf = s.crf.clamp(lo, hi);
            match s.rate_control {
                RateControl::Cbr | RateControl::Vbr if s.target_bitrate_bps.unwrap_or(0) == 0 => {
                    return Err("Set a target bitrate.".into());
                }
                RateControl::TargetSize if s.target_size_bytes.unwrap_or(0) == 0 => {
                    return Err("Set a target file size.".into());
                }
                _ => {}
            }
            Resolved::Encode { spec: s, platform: None }
        }
    })
}

/// Even dimensions; for a locked resize the height is the one ffmpeg will derive.
fn normalized_resize(r: ResizeSpec, pw: u32, ph: u32) -> ResizeSpec {
    let r = r.with_even_dimensions();
    if r.aspect_locked {
        ResizeSpec { width: r.width, height: derived_even_height(r.width, pw, ph), aspect_locked: true }
    } else {
        r
    }
}

/// Final frame size produced by `resize` on a `pw`×`ph` input.
fn resize_output(r: ResizeSpec, pw: u32, ph: u32) -> (u32, u32) {
    let r = r.with_even_dimensions();
    if r.aspect_locked {
        (r.width, derived_even_height(r.width, pw, ph))
    } else {
        (r.width, r.height)
    }
}

/// Locked resize that scales `pw`×`ph` by `factor` (width even, height derived).
fn scaled_resize(pw: u32, ph: u32, factor: f64) -> ResizeSpec {
    let w = round_even(((pw as f64 * factor).round() as u32).max(2));
    ResizeSpec { width: w, height: derived_even_height(w, pw, ph), aspect_locked: true }
}

/// Largest factor (≤ 1) that fits `w`×`h` in the box, orientation-agnostic: the long side
/// must fit the box's long side and the short side its short side.
fn box_factor(w: u32, h: u32, p: &PlatformPreset) -> f64 {
    let (Some(bw), Some(bh)) = (p.max_width, p.max_height) else { return 1.0 };
    if w == 0 || h == 0 {
        return 1.0;
    }
    let (long, short) = (w.max(h) as f64, w.min(h) as f64);
    let (blong, bshort) = (bw.max(bh) as f64, bw.min(bh) as f64);
    (blong / long).min(bshort / short).min(1.0)
}

struct PlatformOutcome {
    resize: Option<ResizeSpec>,
    /// Chosen video bitrate when the preset has a size cap.
    video_bps: Option<u64>,
    warnings: Vec<String>,
}

#[allow(clippy::too_many_arguments)]
fn resolve_platform(
    p: &PlatformPreset,
    base: &CompressionSpec,
    pw: u32,
    ph: u32,
    fps: f64,
    duration: f64,
    audio_kbps: u32,
    user_resize: Option<ResizeSpec>,
) -> Result<PlatformOutcome, String> {
    let mut warnings = Vec::new();

    if p.max_width.zip(p.max_height).is_some_and(|(bw, bh)| bh > bw) && ph <= pw {
        warnings.push("Tip: crop to 9:16".to_string());
    }

    let budget = match p.max_size_bytes {
        Some(cap) => {
            if !is_positive(duration) {
                return Err("Can't fit a size limit: the clip duration is unknown.".into());
            }
            Some(compute_target_bitrate(cap, duration, audio_kbps, SAFETY_MARGIN))
        }
        None => None,
    };
    let bitrate_for = |w: u32, h: u32| -> Option<u64> {
        budget.map(|b| {
            let mut v = b.min(quality_ceiling_bps(w.min(h), fps));
            if let Some(extra) = base.target_bitrate_bps.filter(|x| *x > 0) {
                v = v.min(extra);
            }
            v
        })
    };
    let bpp = |bps: u64, w: u32, h: u32| bps as f64 / (w as f64 * h as f64 * fps).max(1.0);
    let long_warning = || match p.max_size_bytes {
        Some(cap) => format!("Clip is long for {} — consider cutting it", format_mb(cap)),
        None => String::new(),
    };

    if pw == 0 || ph == 0 {
        return Ok(PlatformOutcome { resize: user_resize, video_bps: bitrate_for(pw.max(1), ph.max(1)), warnings });
    }

    // ---- a user resize wins, but is shrunk to the box and never upscales
    if let Some(r) = user_resize {
        let (ow, oh) = resize_output(r, pw, ph);
        let up = (pw as f64 / ow as f64).min(ph as f64 / oh as f64);
        let bx = box_factor(ow, oh, p);
        let f = bx.min(up);
        let resize = if f >= 1.0 {
            Some(r)
        } else {
            let fitted = if r.aspect_locked {
                let w = round_even(((ow as f64 * f).floor() as u32).max(2));
                ResizeSpec { width: w, height: derived_even_height(w, pw, ph), aspect_locked: true }
            } else {
                ResizeSpec {
                    width: round_even(((ow as f64 * f).floor() as u32).max(2)),
                    height: round_even(((oh as f64 * f).floor() as u32).max(2)),
                    aspect_locked: false,
                }
            };
            let (fw, fh) = resize_output(fitted, pw, ph);
            if bx < up {
                warnings.push(format!("Resize reduced to {fw}×{fh} to fit {}", p.name));
            }
            if (fw, fh) == (pw, ph) || (fitted.aspect_locked && fw >= pw) {
                None
            } else {
                Some(fitted)
            }
        };
        let (fw, fh) = resize.map(|r| resize_output(r, pw, ph)).unwrap_or((pw, ph));
        let video_bps = bitrate_for(fw, fh);
        if let Some(bps) = video_bps {
            if bpp(bps, fw, fh) < MIN_BITS_PER_PIXEL {
                warnings.push(long_warning());
            }
        }
        return Ok(PlatformOutcome { resize, video_bps, warnings });
    }

    // ---- automatic: native (box-fitted), then smaller ladder rungs
    let short = pw.min(ph);
    let top = box_factor(pw, ph, p);
    let mut candidates: Vec<Option<ResizeSpec>> = Vec::new();
    candidates.push(if top >= 1.0 { None } else { Some(scaled_resize(pw, ph, top)) });
    let top_short = (short as f64 * top).floor() as u32;
    for rung in RESOLUTION_LADDER {
        if rung < top_short {
            candidates.push(Some(scaled_resize(pw, ph, rung as f64 / short as f64)));
        }
    }
    let dims = |c: &Option<ResizeSpec>| c.map(|r| resize_output(r, pw, ph)).unwrap_or((pw, ph));

    if budget.is_none() {
        return Ok(PlatformOutcome { resize: candidates[0], video_bps: None, warnings });
    }

    let chosen = candidates.iter().find(|c| {
        let (w, h) = dims(c);
        bitrate_for(w, h).is_some_and(|bps| bpp(bps, w, h) >= MIN_BITS_PER_PIXEL)
    });
    let resize = match chosen {
        Some(c) => *c,
        None => {
            warnings.push(long_warning());
            // Smallest candidate at or above the fallback rung, else the smallest there is.
            candidates
                .iter()
                .rev()
                .find(|c| {
                    let (w, h) = dims(c);
                    w.min(h) + 2 >= FALLBACK_SHORT_SIDE
                })
                .copied()
                .unwrap_or(*candidates.last().unwrap())
        }
    };
    let (w, h) = dims(&resize);
    Ok(PlatformOutcome { resize, video_bps: bitrate_for(w, h), warnings })
}

/// Container extension (no dot) for the mode and codecs.
pub fn output_extension(info: &VideoInfo, spec: Option<&CompressionSpec>, mode: OutputMode) -> String {
    if mode == OutputMode::AudioOnly {
        let codec = spec.map(|s| s.audio_codec).unwrap_or(AudioCodec::Copy);
        return match codec {
            AudioCodec::Aac => "m4a".into(),
            AudioCodec::Opus => "opus".into(),
            AudioCodec::Mp3 => "mp3".into(),
            AudioCodec::Copy => {
                let src = info.audio_streams.first().map(|a| a.codec.to_ascii_lowercase()).unwrap_or_default();
                match src.as_str() {
                    "aac" => "m4a",
                    "mp3" => "mp3",
                    "opus" => "opus",
                    "flac" => "flac",
                    _ => "mka",
                }
                .into()
            }
        };
    }
    match spec {
        Some(s) => s
            .container
            .as_deref()
            .map(|c| c.trim().trim_start_matches('.'))
            .filter(|c| !c.is_empty())
            .map(|c| c.to_ascii_lowercase())
            .unwrap_or_else(|| s.codec.default_container().to_string()),
        None => Path::new(&info.path)
            .extension()
            .map(|e| e.to_string_lossy().into_owned())
            .filter(|e| !e.is_empty())
            .unwrap_or_else(|| "mkv".to_string()),
    }
}

fn bits_to_bytes(bps: u64, duration: f64) -> u64 {
    (bps as f64 * duration / 8.0).round() as u64
}

fn estimate_size(info: &VideoInfo, spec: Option<&CompressionSpec>, mode: OutputMode, duration: f64, audio_kbps: &dyn Fn(&CompressionSpec) -> u32) -> Option<u64> {
    if !is_positive(duration) {
        return None;
    }
    let source_audio_bps = info.audio_streams.first().and_then(|a| a.bitrate_bps);
    match spec {
        None => match mode {
            OutputMode::AudioOnly => source_audio_bps.map(|b| bits_to_bytes(b, duration)),
            _ => {
                let size = info.file_size_bytes?;
                if !is_positive(info.duration_sec) {
                    return None;
                }
                let mut est = (size as f64 * (duration / info.duration_sec).min(1.0)).round() as u64;
                if mode == OutputMode::VideoOnly {
                    if let Some(a) = source_audio_bps {
                        est = est.saturating_sub(bits_to_bytes(a, duration));
                    }
                }
                Some(est)
            }
        },
        Some(s) => {
            let audio_bps = audio_kbps(s) as u64 * 1000;
            if mode == OutputMode::AudioOnly {
                if s.audio_codec == AudioCodec::Copy {
                    return source_audio_bps.map(|b| bits_to_bytes(b, duration));
                }
                return Some(bits_to_bytes(s.audio_bitrate_kbps as u64 * 1000, duration));
            }
            match s.rate_control {
                RateControl::Crf => None,
                RateControl::Cbr | RateControl::Vbr | RateControl::TargetSize => {
                    s.target_bitrate_bps.map(|v| bits_to_bytes(v + audio_bps, duration))
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn build_summary(
    info: &VideoInfo,
    spec: Option<&CompressionSpec>,
    mode: OutputMode,
    has_cut: bool,
    draft_had_filters: bool,
    crop: Option<CropSpec>,
    resize: Option<ResizeSpec>,
    dims: (Option<u32>, Option<u32>),
    two_pass: bool,
    estimate: Option<u64>,
) -> String {
    let mut s = String::new();
    if mode == OutputMode::AudioOnly {
        s.push_str("Output: audio only");
        match spec {
            None => s.push_str(" (stream copy)"),
            Some(c) if c.audio_codec == AudioCodec::Copy => s.push_str(" (audio copy)"),
            Some(c) => s.push_str(&format!(" — {} @ {} kbps", c.audio_codec.display_name(), c.audio_bitrate_kbps)),
        }
        if has_cut {
            s.push_str(", cut applied");
        }
        if draft_had_filters {
            s.push_str(" — crop/resize not applicable");
        }
        return s;
    }

    let Some(c) = spec else {
        s.push_str("Output: stream copy (no re-encode)");
        if mode == OutputMode::VideoOnly {
            s.push_str(", video only");
        }
        if has_cut {
            s.push_str(", cut applied");
        }
        if draft_had_filters {
            s.push_str(" — crop/resize ignored");
        }
        return s;
    };

    let (w, h) = (dims.0.unwrap_or(info.width), dims.1.unwrap_or(info.height));
    s.push_str(&format!("Output: {w}×{h} {}", c.codec.display_name()));
    if mode == OutputMode::VideoOnly {
        s.push_str(" (video only)");
    }
    if let Some(cr) = crop {
        s.push_str(&format!(" — crop {}×{} from {}×{}", cr.width, cr.height, info.width, info.height));
    }
    if resize.is_some() {
        if crop.is_some() {
            s.push_str(&format!(", then scale to {w}×{h}"));
        } else {
            s.push_str(&format!(" — scale to {w}×{h}"));
        }
    }
    if two_pass {
        match estimate {
            Some(e) => s.push_str(&format!(" · two-pass to {}", format_mb(e))),
            None => s.push_str(" · two-pass"),
        }
    }
    s
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::presets::catalog;
    use std::path::PathBuf;

    fn audio(codec: &str, bitrate: Option<u64>) -> AudioStreamInfo {
        AudioStreamInfo { index: 1, codec: codec.into(), channels: Some(2), sample_rate: Some(48000), bitrate_bps: bitrate, language: None }
    }

    fn src(w: u32, h: u32, fps: f64, dur: f64) -> VideoInfo {
        VideoInfo {
            path: "C:/videos/clip.mp4".into(),
            duration_sec: dur,
            width: w,
            height: h,
            sar_num: 1,
            sar_den: 1,
            video_codec: "h264".into(),
            profile: Some("High".into()),
            pix_fmt: Some("yuv420p".into()),
            fps,
            bitrate_bps: Some(10_000_000),
            audio_streams: vec![audio("aac", Some(192_000))],
            container_format: "mov,mp4,m4a,3gp,3g2,mj2".into(),
            file_size_bytes: Some(100_000_000),
        }
    }

    fn hd() -> VideoInfo {
        src(1920, 1080, 30.0, 100.0)
    }

    fn draft(compression: CompressionChoice) -> JobDraft {
        JobDraft { cut: None, crop: None, resize: None, compression, mode: OutputMode::Everything, output_dir: None }
    }

    fn quality(id: &str) -> CompressionChoice {
        CompressionChoice::Quality { preset_id: id.into() }
    }

    fn platform(id: &str) -> CompressionChoice {
        CompressionChoice::Platform { platform_id: id.into() }
    }

    fn custom(spec: CompressionSpec) -> CompressionChoice {
        CompressionChoice::Custom { spec }
    }

    fn base_spec() -> CompressionSpec {
        catalog().quality_or_default("web-medium").unwrap().spec.clone()
    }

    fn run(info: &VideoInfo, d: &JobDraft) -> JobPlan {
        plan(info, d, &catalog(), |_| false).unwrap()
    }

    fn crop(x: u32, y: u32, w: u32, h: u32) -> CropSpec {
        CropSpec { x, y, width: w, height: h }
    }

    fn rs(w: u32, h: u32, locked: bool) -> ResizeSpec {
        ResizeSpec { width: w, height: h, aspect_locked: locked }
    }

    fn cut(a: f64, b: f64) -> CutSpec {
        CutSpec { start_sec: a, end_sec: b, accurate: false }
    }

    fn file_name(p: &str) -> String {
        PathBuf::from(p).file_name().unwrap().to_string_lossy().into_owned()
    }

    // ---- quality presets & summaries

    #[test]
    fn default_quality_plain_summary() {
        let p = run(&hd(), &draft(quality("web-medium")));
        assert_eq!(p.summary, "Output: 1920×1080 H.264");
        assert_eq!((p.output_width, p.output_height), (Some(1920), Some(1080)));
        assert_eq!(p.spec.as_ref().unwrap().crf, 23);
        assert!(!p.two_pass);
        assert_eq!(p.estimated_size_bytes, None);
        assert!(p.warnings.is_empty());
        assert_eq!(p.output_path, Path::new("C:/videos").join("clip_VideoCrop.mp4").to_string_lossy());
    }

    #[test]
    fn crop_then_scale_summary() {
        let mut d = draft(quality("web-medium"));
        d.crop = Some(crop(0, 0, 1600, 900));
        d.resize = Some(rs(1280, 720, true));
        let p = run(&hd(), &d);
        assert_eq!(p.summary, "Output: 1280×720 H.264 — crop 1600×900 from 1920×1080, then scale to 1280×720");
        assert_eq!(p.crop, Some(crop(0, 0, 1600, 900)));
        assert_eq!(p.resize, Some(rs(1280, 720, true)));
        assert_eq!((p.output_width, p.output_height), (Some(1280), Some(720)));
    }

    #[test]
    fn crop_only_summary() {
        let mut d = draft(quality("high-slow"));
        d.crop = Some(crop(100, 50, 800, 600));
        let p = run(&hd(), &d);
        assert_eq!(p.summary, "Output: 800×600 AV1 — crop 800×600 from 1920×1080");
        assert_eq!(p.resize, None);
    }

    #[test]
    fn resize_only_summary_unlocked_and_odd() {
        let mut d = draft(quality("medium-fast"));
        d.resize = Some(rs(1281, 721, false));
        let p = run(&hd(), &d);
        assert_eq!(p.summary, "Output: 1280×720 H.265 — scale to 1280×720");
        assert_eq!(p.resize, Some(rs(1280, 720, false)));
    }

    #[test]
    fn locked_resize_height_uses_post_crop_aspect() {
        let mut d = draft(quality("web-medium"));
        d.crop = Some(crop(0, 0, 1080, 1080));
        d.resize = Some(rs(500, 999, true));
        let p = run(&hd(), &d);
        assert_eq!((p.output_width, p.output_height), (Some(500), Some(500)));
        assert_eq!(p.resize, Some(rs(500, 500, true)));
    }

    #[test]
    fn video_only_summary() {
        let mut d = draft(quality("web-medium"));
        d.mode = OutputMode::VideoOnly;
        let p = run(&hd(), &d);
        assert_eq!(p.summary, "Output: 1920×1080 H.264 (video only)");
    }

    #[test]
    fn unknown_quality_falls_back_to_default() {
        let p = run(&hd(), &draft(quality("does-not-exist")));
        assert_eq!(p.spec.unwrap().crf, 23);
    }

    #[test]
    fn crop_equal_to_source_is_dropped_and_odd_crop_clamped() {
        let mut d = draft(quality("web-medium"));
        d.crop = Some(crop(0, 0, 1920, 1080));
        assert_eq!(run(&hd(), &d).crop, None);
        d.crop = Some(crop(0, 0, 5000, 5000));
        assert_eq!(run(&hd(), &d).crop, None);
        d.crop = Some(crop(3, 5, 101, 77));
        let p = run(&hd(), &d);
        assert_eq!(p.crop, Some(crop(4, 6, 100, 76)));
        assert_eq!((p.output_width, p.output_height), (Some(100), Some(76)));
    }

    #[test]
    fn resize_too_small_errors() {
        let mut d = draft(quality("web-medium"));
        d.resize = Some(rs(1, 100, false));
        assert!(plan(&hd(), &d, &catalog(), |_| false).is_err());
        d.resize = Some(rs(100, 0, false));
        assert!(plan(&hd(), &d, &catalog(), |_| false).is_err());
        d.resize = Some(rs(100, 0, true));
        assert!(plan(&hd(), &d, &catalog(), |_| false).is_ok());
    }

    #[test]
    fn non_platform_resize_may_upscale() {
        let mut d = draft(quality("web-medium"));
        d.resize = Some(rs(3840, 0, true));
        let p = run(&hd(), &d);
        assert_eq!((p.output_width, p.output_height), (Some(3840), Some(2160)));
    }

    // ---- stream copy

    #[test]
    fn stream_copy_ignores_filters() {
        let mut d = draft(CompressionChoice::Off);
        d.cut = Some(cut(10.0, 20.0));
        d.crop = Some(crop(0, 0, 100, 100));
        d.resize = Some(rs(64, 64, false));
        let p = run(&hd(), &d);
        assert_eq!(p.summary, "Output: stream copy (no re-encode), cut applied — crop/resize ignored");
        assert_eq!(p.spec, None);
        assert_eq!(p.crop, None);
        assert_eq!(p.resize, None);
        assert!(!p.two_pass);
        assert_eq!((p.output_width, p.output_height), (Some(1920), Some(1080)));
        // source extension kept; 10% of a 100 MB file
        assert_eq!(file_name(&p.output_path), "clip_VideoCrop.mp4");
        assert_eq!(p.estimated_size_bytes, Some(10_000_000));
    }

    #[test]
    fn stream_copy_plain_and_video_only() {
        let p = run(&hd(), &draft(CompressionChoice::Off));
        assert_eq!(p.summary, "Output: stream copy (no re-encode)");
        assert_eq!(p.estimated_size_bytes, Some(100_000_000));
        let mut d = draft(CompressionChoice::Off);
        d.mode = OutputMode::VideoOnly;
        let p = run(&hd(), &d);
        assert_eq!(p.summary, "Output: stream copy (no re-encode), video only");
        assert_eq!(p.estimated_size_bytes, Some(100_000_000 - 2_400_000));
    }

    #[test]
    fn stream_copy_mkv_source_keeps_mkv() {
        let mut info = hd();
        info.path = "C:/videos/obs.mkv".into();
        let p = run(&info, &draft(CompressionChoice::Off));
        assert_eq!(file_name(&p.output_path), "obs_VideoCrop.mkv");
    }

    // ---- audio only

    #[test]
    fn audio_only_reencode() {
        let mut d = draft(quality("web-medium"));
        d.mode = OutputMode::AudioOnly;
        d.crop = Some(crop(0, 0, 100, 100));
        d.cut = Some(cut(0.0, 10.0));
        let p = run(&hd(), &d);
        assert_eq!(p.summary, "Output: audio only — AAC @ 128 kbps, cut applied — crop/resize not applicable");
        assert_eq!(file_name(&p.output_path), "clip_VideoCrop.m4a");
        assert_eq!((p.output_width, p.output_height), (None, None));
        assert_eq!(p.crop, None);
        assert_eq!(p.estimated_size_bytes, Some(160_000));
        assert!(!p.two_pass);
    }

    #[test]
    fn audio_only_extensions() {
        let mut d = draft(quality("high-fast"));
        d.mode = OutputMode::AudioOnly;
        assert_eq!(file_name(&run(&hd(), &d).output_path), "clip_VideoCrop.opus");
        let mut s = base_spec();
        s.audio_codec = AudioCodec::Mp3;
        d.compression = custom(s.clone());
        assert_eq!(file_name(&run(&hd(), &d).output_path), "clip_VideoCrop.mp3");
        s.audio_codec = AudioCodec::Copy;
        d.compression = custom(s);
        let p = run(&hd(), &d);
        assert_eq!(file_name(&p.output_path), "clip_VideoCrop.m4a");
        assert_eq!(p.summary, "Output: audio only (audio copy)");
        d.compression = CompressionChoice::Off;
        let mut info = hd();
        info.audio_streams = vec![audio("ac3", None)];
        let p = run(&info, &d);
        assert_eq!(file_name(&p.output_path), "clip_VideoCrop.mka");
        assert_eq!(p.summary, "Output: audio only (stream copy)");
        assert_eq!(p.estimated_size_bytes, None);
    }

    #[test]
    fn audio_only_without_audio_errors() {
        let mut info = hd();
        info.audio_streams.clear();
        let mut d = draft(quality("web-medium"));
        d.mode = OutputMode::AudioOnly;
        assert!(plan(&info, &d, &catalog(), |_| false).is_err());
    }

    #[test]
    fn audio_only_platform_is_single_pass() {
        let mut d = draft(platform("discord"));
        d.mode = OutputMode::AudioOnly;
        let p = run(&hd(), &d);
        assert!(!p.two_pass);
        assert_eq!(file_name(&p.output_path), "clip_VideoCrop.m4a");
        assert!(p.summary.starts_with("Output: audio only — AAC @ 128 kbps"));
    }

    // ---- output path

    #[test]
    fn output_dir_and_collisions() {
        let mut d = draft(quality("web-medium"));
        d.output_dir = Some("D:/exports".into());
        let p = plan(&hd(), &d, &catalog(), |path| file_name(&path.to_string_lossy()) == "clip_VideoCrop.mp4").unwrap();
        assert_eq!(p.output_path, Path::new("D:/exports").join("clip_VideoCrop2.mp4").to_string_lossy());
        d.output_dir = Some("   ".into());
        let p = run(&hd(), &d);
        assert!(p.output_path.starts_with("C:/videos"));
    }

    #[test]
    fn container_resolution() {
        let mut s = base_spec();
        s.codec = VideoCodec::Vp9;
        s.audio_codec = AudioCodec::Opus;
        assert_eq!(file_name(&run(&hd(), &draft(custom(s.clone()))).output_path), "clip_VideoCrop.webm");
        s.container = Some(".MKV".into());
        assert_eq!(file_name(&run(&hd(), &draft(custom(s.clone()))).output_path), "clip_VideoCrop.mkv");
        s.container = Some("".into());
        assert_eq!(file_name(&run(&hd(), &draft(custom(s))).output_path), "clip_VideoCrop.webm");
        assert_eq!(file_name(&run(&hd(), &draft(quality("low-slow"))).output_path), "clip_VideoCrop.mp4");
    }

    // ---- cut

    #[test]
    fn full_range_cut_is_dropped() {
        let mut d = draft(quality("web-medium"));
        d.cut = Some(cut(0.0, 100.0));
        assert_eq!(run(&hd(), &d).cut, None);
        d.cut = Some(cut(-1.0, 99.9995));
        assert_eq!(run(&hd(), &d).cut, None);
        d.cut = Some(cut(0.0, 150.0));
        assert_eq!(run(&hd(), &d).cut, None);
    }

    #[test]
    fn partial_cut_is_kept_and_clamped() {
        let mut d = draft(quality("web-medium"));
        d.cut = Some(CutSpec { start_sec: 5.0, end_sec: 500.0, accurate: true });
        assert_eq!(run(&hd(), &d).cut, Some(CutSpec { start_sec: 5.0, end_sec: 100.0, accurate: true }));
        d.cut = Some(cut(0.0, 50.0));
        assert_eq!(run(&hd(), &d).cut, Some(cut(0.0, 50.0)));
    }

    #[test]
    fn too_short_cut_errors() {
        let mut d = draft(quality("web-medium"));
        d.cut = Some(cut(5.0, 5.05));
        assert!(plan(&hd(), &d, &catalog(), |_| false).is_err());
        d.cut = Some(cut(5.0, 4.0));
        assert!(plan(&hd(), &d, &catalog(), |_| false).is_err());
        d.cut = Some(cut(5.0, 5.1));
        assert!(plan(&hd(), &d, &catalog(), |_| false).is_ok());
        d.cut = Some(cut(f64::NAN, 5.0));
        assert!(plan(&hd(), &d, &catalog(), |_| false).is_err());
    }

    // ---- custom rate control

    #[test]
    fn custom_target_size_computes_bitrate() {
        let mut s = base_spec();
        s.rate_control = RateControl::TargetSize;
        s.target_size_bytes = Some(25 * 1024 * 1024);
        let mut d = draft(custom(s));
        d.cut = Some(cut(10.0, 30.0));
        let p = run(&hd(), &d);
        assert!(p.two_pass);
        let bps = p.spec.as_ref().unwrap().target_bitrate_bps.unwrap();
        assert_eq!(bps, compute_target_bitrate(25 * 1024 * 1024, 20.0, 128, 0.97));
        let est = p.estimated_size_bytes.unwrap();
        let cap = 25 * 1024 * 1024;
        assert!(est <= cap && est as f64 > cap as f64 * 0.95, "{est}");
        assert!(p.summary.ends_with(&format!(" · two-pass to {}", format_mb(est))), "{}", p.summary);
    }

    #[test]
    fn custom_target_size_video_only_reserves_no_audio() {
        let mut s = base_spec();
        s.rate_control = RateControl::TargetSize;
        s.target_size_bytes = Some(10_000_000);
        let mut d = draft(custom(s));
        d.mode = OutputMode::VideoOnly;
        d.cut = Some(cut(0.0, 10.0));
        let p = run(&hd(), &d);
        assert_eq!(p.spec.unwrap().target_bitrate_bps, Some(7_760_000));
    }

    #[test]
    fn custom_target_size_audio_copy_uses_source_bitrate() {
        let mut s = base_spec();
        s.rate_control = RateControl::TargetSize;
        s.target_size_bytes = Some(10_000_000);
        s.audio_codec = AudioCodec::Copy;
        let mut d = draft(custom(s));
        d.cut = Some(cut(0.0, 10.0));
        let p = run(&hd(), &d);
        assert_eq!(p.spec.unwrap().target_bitrate_bps, Some(7_760_000 - 192_000));
    }

    #[test]
    fn custom_missing_targets_error() {
        let mut s = base_spec();
        s.rate_control = RateControl::TargetSize;
        assert!(plan(&hd(), &draft(custom(s.clone())), &catalog(), |_| false).is_err());
        s.rate_control = RateControl::Vbr;
        assert!(plan(&hd(), &draft(custom(s.clone())), &catalog(), |_| false).is_err());
        s.rate_control = RateControl::Cbr;
        s.target_bitrate_bps = Some(0);
        assert!(plan(&hd(), &draft(custom(s)), &catalog(), |_| false).is_err());
    }

    #[test]
    fn custom_vbr_estimate() {
        let mut s = base_spec();
        s.rate_control = RateControl::Vbr;
        s.target_bitrate_bps = Some(2_000_000);
        let p = run(&hd(), &draft(custom(s)));
        // (2 Mbps + 128 kbps) * 100 s / 8
        assert_eq!(p.estimated_size_bytes, Some(26_600_000));
        assert!(!p.two_pass);
        assert_eq!(p.summary, "Output: 1920×1080 H.264");
    }

    #[test]
    fn custom_crf_is_clamped_to_codec_range() {
        let mut s = base_spec();
        s.crf = 99;
        assert_eq!(run(&hd(), &draft(custom(s.clone()))).spec.unwrap().crf, 51);
        s.codec = VideoCodec::Av1;
        assert_eq!(run(&hd(), &draft(custom(s))).spec.unwrap().crf, 63);
    }

    #[test]
    fn target_size_with_unknown_duration_errors() {
        let mut s = base_spec();
        s.rate_control = RateControl::TargetSize;
        s.target_size_bytes = Some(1_000_000);
        let info = src(1920, 1080, 30.0, 0.0);
        assert!(plan(&info, &draft(custom(s)), &catalog(), |_| false).is_err());
        assert!(plan(&info, &draft(platform("discord")), &catalog(), |_| false).is_err());
    }

    // ---- platform presets

    #[test]
    fn unknown_platform_errors() {
        assert!(plan(&hd(), &draft(platform("myspace")), &catalog(), |_| false).is_err());
    }

    #[test]
    fn discord_long_1080p60_clip_drops_to_360p_and_warns() {
        let info = src(1920, 1080, 60.0, 180.0);
        let p = run(&info, &draft(platform("discord")));
        assert!(p.two_pass);
        assert_eq!((p.output_width, p.output_height), (Some(640), Some(360)));
        assert_eq!(p.resize, Some(rs(640, 360, true)));
        assert_eq!(p.warnings, ["Clip is long for 10 MB — consider cutting it"]);
        let s = p.spec.as_ref().unwrap();
        assert_eq!(s.rate_control, RateControl::TargetSize);
        assert_eq!(s.target_size_bytes, Some(10_000_000));
        assert_eq!(s.target_bitrate_bps, Some(compute_target_bitrate(10_000_000, 180.0, 128, 0.97)));
        assert_eq!(s.speed, SpeedPreset::Medium);
        assert_eq!(s.pixel_format, PixelFormat::Yuv420p);
        assert_eq!(p.summary, "Output: 640×360 H.264 — scale to 640×360 · two-pass to 9.7 MB");
    }

    #[test]
    fn discord_one_minute_1080p30_picks_540p() {
        let info = src(1920, 1080, 30.0, 60.0);
        let p = run(&info, &draft(platform("discord")));
        assert_eq!((p.output_width, p.output_height), (Some(960), Some(540)));
        assert!(p.warnings.is_empty());
    }

    #[test]
    fn discord_short_clip_keeps_1080p() {
        let mut d = draft(platform("discord"));
        d.cut = Some(cut(0.0, 10.0));
        let p = run(&hd(), &d);
        assert_eq!(p.resize, None);
        assert_eq!((p.output_width, p.output_height), (Some(1920), Some(1080)));
        // budget 7.632 Mbps is under the 8 Mbps 1080p30 ceiling
        assert_eq!(p.spec.unwrap().target_bitrate_bps, Some(7_632_000));
        assert_eq!(p.summary, "Output: 1920×1080 H.264 · two-pass to 9.7 MB");
    }

    #[test]
    fn short_clip_bitrate_capped_by_quality_ceiling() {
        let mut d = draft(platform("discord-nitro-basic"));
        d.cut = Some(cut(0.0, 10.0));
        let p = run(&hd(), &d);
        assert_eq!(p.resize, None);
        assert_eq!(p.spec.as_ref().unwrap().target_bitrate_bps, Some(8_000_000));
        // (8 Mbps + 128 kbps) * 10 s / 8
        assert_eq!(p.estimated_size_bytes, Some(10_160_000));
        assert_eq!(p.summary, "Output: 1920×1080 H.264 · two-pass to 10.2 MB");
        // 60 fps raises the ceiling by 1.5x
        let p = run(&src(1920, 1080, 60.0, 100.0), &d);
        assert_eq!(p.spec.unwrap().target_bitrate_bps, Some(12_000_000));
    }

    #[test]
    fn nitro_4k_source_fits_1440_box() {
        let mut d = draft(platform("discord-nitro"));
        d.cut = Some(cut(0.0, 10.0));
        let p = run(&src(3840, 2160, 30.0, 60.0), &d);
        assert_eq!((p.output_width, p.output_height), (Some(2560), Some(1440)));
        assert_eq!(p.resize, Some(rs(2560, 1440, true)));
        assert_eq!(p.spec.unwrap().target_bitrate_bps, Some(12_000_000));
    }

    #[test]
    fn whatsapp_box_limits_to_720p() {
        let mut d = draft(platform("whatsapp"));
        d.cut = Some(cut(0.0, 5.0));
        let p = run(&hd(), &d);
        assert_eq!((p.output_width, p.output_height), (Some(1280), Some(720)));
        let s = p.spec.unwrap();
        assert_eq!(s.audio_bitrate_kbps, 96);
        assert_eq!(s.target_size_bytes, Some(16_000_000));
    }

    #[test]
    fn platform_never_upscales_small_source() {
        let small = src(640, 360, 30.0, 5.0);
        let p = run(&small, &draft(platform("discord")));
        assert_eq!(p.resize, None);
        assert_eq!((p.output_width, p.output_height), (Some(640), Some(360)));
        let p = run(&small, &draft(platform("youtube")));
        assert_eq!(p.resize, None);
        let mut d = draft(platform("discord"));
        d.resize = Some(rs(1280, 720, true));
        let p = run(&small, &d);
        assert_eq!(p.resize, None);
        assert_eq!((p.output_width, p.output_height), (Some(640), Some(360)));
    }

    #[test]
    fn platform_user_resize_wins_but_shrinks_to_box() {
        let mut d = draft(platform("discord"));
        d.cut = Some(cut(0.0, 5.0));
        d.resize = Some(rs(1280, 720, true));
        let p = run(&src(3840, 2160, 30.0, 60.0), &d);
        assert_eq!(p.resize, Some(rs(1280, 720, true)));
        assert!(p.warnings.is_empty());

        d.resize = Some(rs(3840, 2160, false));
        let p = run(&src(3840, 2160, 30.0, 60.0), &d);
        assert_eq!((p.output_width, p.output_height), (Some(1920), Some(1080)));
        assert_eq!(p.resize, Some(rs(1920, 1080, false)));
        assert_eq!(p.warnings, ["Resize reduced to 1920×1080 to fit Discord"]);

        // an unlocked squash keeps its own aspect when shrunk
        d.resize = Some(rs(2000, 2000, false));
        let p = run(&src(3840, 2160, 30.0, 60.0), &d);
        assert_eq!(p.resize, Some(rs(1080, 1080, false)));

        d.resize = Some(rs(3840, 0, true));
        let p = run(&src(3840, 2160, 30.0, 60.0), &d);
        assert_eq!(p.resize, Some(rs(1920, 1080, true)));
    }

    #[test]
    fn platform_user_upscale_is_clamped_to_source() {
        let mut d = draft(platform("youtube"));
        d.resize = Some(rs(1280, 1280, false));
        let p = run(&src(640, 360, 30.0, 5.0), &d);
        // per-axis no-upscale factor min(640/1280, 360/1280)
        assert_eq!(p.resize, Some(rs(360, 360, false)));
        assert!(p.warnings.is_empty());
    }

    #[test]
    fn platform_user_resize_low_bitrate_warns() {
        let mut d = draft(platform("discord"));
        d.resize = Some(rs(1920, 1080, false));
        let p = run(&src(1920, 1080, 60.0, 180.0), &d);
        assert_eq!(p.resize, Some(rs(1920, 1080, false)));
        assert_eq!(p.warnings, ["Clip is long for 10 MB — consider cutting it"]);
    }

    #[test]
    fn platform_with_crop_uses_post_crop_frame() {
        let mut d = draft(platform("discord"));
        d.crop = Some(crop(0, 0, 1600, 900));
        d.cut = Some(cut(0.0, 60.0));
        let p = run(&hd(), &d);
        // 1600x900 fails bpp, 1280x720 fails, 960x540 passes
        assert_eq!((p.output_width, p.output_height), (Some(960), Some(540)));
        assert_eq!(p.summary, "Output: 960×540 H.264 — crop 1600×900 from 1920×1080, then scale to 960×540 · two-pass to 9.7 MB");
    }

    #[test]
    fn portrait_source_uses_short_side_and_rotated_box() {
        let info = src(1080, 1920, 30.0, 60.0);
        let p = run(&info, &draft(platform("discord")));
        assert_eq!((p.output_width, p.output_height), (Some(540), Some(960)));
        let mut d = draft(platform("discord"));
        d.cut = Some(cut(0.0, 5.0));
        let p = run(&info, &d);
        assert_eq!(p.resize, None);
        assert_eq!((p.output_width, p.output_height), (Some(1080), Some(1920)));
    }

    #[test]
    fn video_only_and_silent_sources_reserve_no_audio() {
        let mut d = draft(platform("discord"));
        d.cut = Some(cut(0.0, 10.0));
        d.mode = OutputMode::VideoOnly;
        assert_eq!(run(&hd(), &d).spec.unwrap().target_bitrate_bps, Some(7_760_000));
        let mut info = hd();
        info.audio_streams.clear();
        d.mode = OutputMode::Everything;
        assert_eq!(run(&info, &d).spec.unwrap().target_bitrate_bps, Some(7_760_000));
    }

    #[test]
    fn youtube_uses_crf_without_cap() {
        let p = run(&src(3840, 2160, 60.0, 600.0), &draft(platform("youtube")));
        let s = p.spec.unwrap();
        assert_eq!(s.rate_control, RateControl::Crf);
        assert_eq!((s.crf, s.speed, s.audio_bitrate_kbps), (18, SpeedPreset::Slow, 320));
        assert!(!p.two_pass);
        assert_eq!(p.resize, None);
        assert_eq!(p.estimated_size_bytes, None);
        assert_eq!(p.summary, "Output: 3840×2160 H.264");
    }

    #[test]
    fn vertical_warns_on_landscape_and_fits_box() {
        let p = run(&hd(), &draft(platform("vertical")));
        assert_eq!(p.warnings, ["Tip: crop to 9:16"]);
        assert_eq!(p.spec.as_ref().unwrap().crf, 21);

        let mut d = draft(platform("vertical"));
        d.crop = Some(crop(656, 0, 608, 1080));
        let p = run(&hd(), &d);
        assert!(p.warnings.is_empty());
        assert_eq!((p.output_width, p.output_height), (Some(608), Some(1080)));

        let p = run(&src(2160, 3840, 30.0, 30.0), &draft(platform("vertical")));
        assert!(p.warnings.is_empty());
        assert_eq!((p.output_width, p.output_height), (Some(1080), Some(1920)));
        assert_eq!(p.resize, Some(rs(1080, 1920, true)));
    }

    #[test]
    fn platform_stream_copy_never_happens_but_off_still_works() {
        let mut d = draft(CompressionChoice::Off);
        d.mode = OutputMode::AudioOnly;
        let p = run(&hd(), &d);
        assert_eq!(p.spec, None);
        assert_eq!(p.estimated_size_bytes, Some(2_400_000));
    }

    // ---- helpers

    #[test]
    fn mb_formatting() {
        assert_eq!(format_mb(10_000_000), "10 MB");
        assert_eq!(format_mb(9_700_000), "9.7 MB");
        assert_eq!(format_mb(48_500_000), "48.5 MB");
        assert_eq!(format_mb(485_000_000), "485 MB");
        assert_eq!(format_mb(512_000_000), "512 MB");
    }

    #[test]
    fn ceilings() {
        assert_eq!(quality_ceiling_bps(1080, 30.0), 8_000_000);
        assert_eq!(quality_ceiling_bps(1080, 29.97), 8_000_000);
        assert_eq!(quality_ceiling_bps(1080, 60.0), 12_000_000);
        assert_eq!(quality_ceiling_bps(1440, 30.0), 12_000_000);
        assert_eq!(quality_ceiling_bps(2160, 30.0), 20_000_000);
        assert_eq!(quality_ceiling_bps(720, 30.0), 5_000_000);
        assert_eq!(quality_ceiling_bps(360, 30.0), 2_500_000);
    }

    #[test]
    fn plan_serializes_camel_case() {
        let json = serde_json::to_string(&run(&hd(), &draft(platform("discord")))).unwrap();
        for key in ["outputPath", "outputWidth", "twoPass", "estimatedSizeBytes", "targetBitrateBps"] {
            assert!(json.contains(key), "{key}");
        }
    }

    #[test]
    fn plan_feeds_encoder() {
        let mut d = draft(platform("discord"));
        d.crop = Some(crop(0, 0, 1600, 900));
        d.cut = Some(CutSpec { start_sec: 2.0, end_sec: 62.0, accurate: true });
        let p = run(&hd(), &d);
        let passes = crate::core::encode::build_passes(&p, &hd().path, "C:/tmp/pfx").unwrap();
        assert_eq!(passes.len(), 2);
        assert!(passes[1].contains(&"crop=1600:900:0:0,scale=960:-2:flags=lanczos".to_string()));
        assert_eq!(passes[1].last().unwrap(), &p.output_path);
    }
}
