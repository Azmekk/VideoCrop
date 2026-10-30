//! Preview playback: on-demand HLS (fixed-length segments over a synthetic VOD playlist),
//! still-frame grabs, preview-encoder detection, and the direct-play compatibility hint.

use std::path::Path;

use super::encode::format_seconds;
use super::models::VideoInfo;

/// Every segment is exactly this long except the last.
pub const SEGMENT_SEC: f64 = 4.0;

/// A trailing remainder shorter than this is folded into the previous segment, so float
/// noise (e.g. 8.0000001 s) never produces a near-empty final segment.
const MIN_TAIL_SEC: f64 = 0.001;

/// GOP length large enough that only the forced 4 s keyframes start a GOP.
const PREVIEW_GOP: &str = "600";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewEncoder {
    Nvenc,
    Qsv,
    Amf,
    X264,
}

impl PreviewEncoder {
    /// Detection order: fastest hardware first, libx264 always works.
    pub const ALL: [PreviewEncoder; 4] = [Self::Nvenc, Self::Qsv, Self::Amf, Self::X264];

    pub fn ffmpeg_name(self) -> &'static str {
        match self {
            Self::Nvenc => "h264_nvenc",
            Self::Qsv => "h264_qsv",
            Self::Amf => "h264_amf",
            Self::X264 => "libx264",
        }
    }

    pub fn from_ffmpeg_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.ffmpeg_name() == name)
    }

    pub fn is_hardware(self) -> bool {
        self != Self::X264
    }

    /// Encoder-specific fast / low-latency settings (after `-c:v <name>`).
    fn low_latency_args(self) -> &'static [&'static str] {
        match self {
            Self::Nvenc => &["-preset", "p1", "-tune", "ll", "-rc", "vbr", "-cq", "26", "-forced-idr", "1"],
            Self::Qsv => &["-preset", "veryfast", "-global_quality", "26"],
            Self::Amf => &["-quality", "speed", "-rc", "cqp", "-qp_i", "24", "-qp_p", "26"],
            Self::X264 => &["-preset", "ultrafast", "-tune", "zerolatency", "-crf", "26", "-sc_threshold", "0"],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HlsJobParams {
    pub input_path: String,
    /// First segment this job produces; the job seeks to `start_segment * SEGMENT_SEC`.
    pub start_segment: u32,
    pub encoder: PreviewEncoder,
    /// Output height cap (never upscales), e.g. 720 or 1080.
    pub max_height: u32,
    pub has_audio: bool,
    /// Try hardware decoding (`-hwaccel auto`); the runtime retries without it on failure.
    pub hwaccel: bool,
    /// Directory receiving `seg_<n>.ts` files (written via temp file + rename).
    pub out_dir: String,
}

fn valid_duration(duration_sec: f64) -> Option<f64> {
    (duration_sec.is_finite() && duration_sec > 0.0).then_some(duration_sec)
}

/// `ceil(duration / 4)`, at least 1 (a playlist needs one segment even for an unknown or
/// zero duration).
pub fn segment_count(duration_sec: f64) -> u32 {
    match valid_duration(duration_sec) {
        None => 1,
        Some(d) => {
            let n = ((d - MIN_TAIL_SEC) / SEGMENT_SEC).ceil();
            if n < 1.0 {
                1
            } else if n > u32::MAX as f64 {
                u32::MAX
            } else {
                n as u32
            }
        }
    }
}

/// Duration of segment `n` as listed in the playlist (last segment gets the remainder).
pub fn segment_duration(duration_sec: f64, n: u32) -> f64 {
    let count = segment_count(duration_sec);
    if n + 1 < count {
        return SEGMENT_SEC;
    }
    match valid_duration(duration_sec) {
        None => SEGMENT_SEC,
        Some(d) => (d - SEGMENT_SEC * (count - 1) as f64).max(MIN_TAIL_SEC),
    }
}

/// Source time at which segment `n` starts.
pub fn segment_start_sec(n: u32) -> f64 {
    n as f64 * SEGMENT_SEC
}

/// Segment containing time `t` (clamped to the playlist).
pub fn segment_for_time(duration_sec: f64, t: f64) -> u32 {
    let count = segment_count(duration_sec);
    if !(t.is_finite() && t > 0.0) {
        return 0;
    }
    ((t / SEGMENT_SEC).floor() as u64).min(count as u64 - 1) as u32
}

/// VOD playlist listing every segment as `seg/<n>.ts` (relative to the playlist URL).
pub fn hls_playlist(duration_sec: f64) -> String {
    let count = segment_count(duration_sec);
    let mut s = String::with_capacity(96 + count as usize * 32);
    s.push_str("#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:4\n#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-PLAYLIST-TYPE:VOD\n");
    for n in 0..count {
        s.push_str(&format!("#EXTINF:{:.6},\nseg/{n}.ts\n", segment_duration(duration_sec, n)));
    }
    s.push_str("#EXT-X-ENDLIST\n");
    s
}

/// File name the HLS job writes for segment `n` inside `out_dir`.
pub fn segment_file_name(n: u32) -> String {
    format!("seg_{n}.ts")
}

/// The ffmpeg playlist file the HLS job writes (ignored by the protocol, which serves the
/// synthetic playlist).
pub const JOB_PLAYLIST_NAME: &str = "job.m3u8";

fn join(dir: &str, file: &str) -> String {
    Path::new(dir).join(file).to_string_lossy().into_owned()
}

/// `scale=-2:trunc(min(ih\,H)/2)*2,format=yuv420p` — even dimensions, never upscales.
/// The `\,` is filtergraph-level escaping (not shell quoting), valid in an argument list.
pub fn preview_filter(max_height: u32) -> String {
    let h = max_height.max(2);
    format!("scale=-2:trunc(min(ih\\,{h})/2)*2,format=yuv420p")
}

pub fn hls_job_args(p: &HlsJobParams) -> Vec<String> {
    let start = segment_start_sec(p.start_segment);
    let start_str = format_seconds(start);
    let mut a: Vec<String> = Vec::with_capacity(64);
    let mut push = |items: &[&str]| a.extend(items.iter().map(|s| s.to_string()));

    push(&["-hide_banner", "-nostats", "-loglevel", "error"]);
    if p.hwaccel {
        push(&["-hwaccel", "auto"]);
    }
    push(&["-ss", &start_str, "-i", &p.input_path]);
    push(&["-map", "0:v:0"]);
    if p.has_audio {
        push(&["-map", "0:a:0"]);
    }
    push(&["-vf", &preview_filter(p.max_height)]);
    push(&["-c:v", p.encoder.ffmpeg_name()]);
    push(p.encoder.low_latency_args());
    push(&["-g", PREVIEW_GOP, "-force_key_frames", "expr:gte(t,n_forced*4)"]);
    if p.has_audio {
        push(&["-c:a", "aac", "-ac", "2", "-b:a", "160k"]);
    }
    push(&[
        "-f", "hls",
        "-hls_time", "4",
        "-hls_segment_type", "mpegts",
        "-hls_flags", "temp_file+independent_segments",
        "-hls_list_size", "0",
        "-start_number", &p.start_segment.to_string(),
        "-output_ts_offset", &start_str,
        "-hls_segment_filename", &join(&p.out_dir, "seg_%d.ts"),
        &join(&p.out_dir, JOB_PLAYLIST_NAME),
    ]);
    a
}

/// Tiny synthetic encode (`-f lavfi color ... -f null -`) used to test whether an encoder works.
pub fn encoder_probe_args(encoder: PreviewEncoder) -> Vec<String> {
    ["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", "color=c=black:s=256x144:d=0.2", "-c:v", encoder.ffmpeg_name(), "-f", "null", "-"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Single JPEG to stdout. `fast` = keyframe-snapped (`-noaccurate_seek`, `-skip_frame nokey`)
/// for live scrubbing. `width` None = native resolution (used by the crop editor).
pub fn frame_args(input_path: &str, time_sec: f64, width: Option<u32>, fast: bool) -> Vec<String> {
    let t = if time_sec.is_finite() && time_sec > 0.0 { time_sec } else { 0.0 };
    let mut a: Vec<String> = Vec::with_capacity(24);
    let mut push = |items: &[&str]| a.extend(items.iter().map(|s| s.to_string()));
    push(&["-hide_banner", "-loglevel", "error"]);
    if fast {
        push(&["-noaccurate_seek", "-skip_frame", "nokey"]);
    }
    push(&["-ss", &format_seconds(t), "-i", input_path, "-frames:v", "1"]);
    if let Some(w) = width.filter(|w| *w > 0) {
        push(&["-vf", &format!("scale={w}:-2")]);
    }
    push(&["-f", "image2pipe", "-c:v", "mjpeg", "-q:v", "3", "pipe:1"]);
    a
}

/// Heuristic for whether WebView2 can play the original file directly.
/// HEVC is always false here: support is per-machine, the frontend checks `canPlayType`.
pub fn direct_likely_playable(info: &VideoInfo) -> bool {
    let fmt = info.container_format.to_ascii_lowercase();
    let path = info.path.to_ascii_lowercase();
    let container_ok = fmt.contains("mp4") || fmt.contains("mov") || (fmt.contains("webm") && path.ends_with(".webm"));
    if !container_ok {
        return false;
    }

    let video_ok = match info.video_codec.to_ascii_lowercase().as_str() {
        "h264" => {
            let pix_ok = match info.pix_fmt.as_deref() {
                None => true,
                Some(p) => matches!(p, "yuv420p" | "yuvj420p"),
            };
            let profile_ok = match info.profile.as_deref() {
                None => true,
                Some(p) => !(p.contains("4:4:4") || p.contains("10") || p.contains("4:2:2")),
            };
            pix_ok && profile_ok
        }
        "vp8" | "vp9" | "av1" => true,
        _ => false,
    };
    if !video_ok {
        return false;
    }

    match info.audio_streams.first() {
        None => true,
        Some(a) => matches!(a.codec.to_ascii_lowercase().as_str(), "aac" | "mp3" | "opus" | "vorbis" | "flac"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::AudioStreamInfo;

    // ---- playlist math

    fn extinfs(pl: &str) -> Vec<String> {
        pl.lines().filter(|l| l.starts_with("#EXTINF:")).map(str::to_string).collect()
    }

    #[test]
    fn playlist_exact_multiple() {
        let pl = hls_playlist(8.0);
        assert_eq!(segment_count(8.0), 2);
        assert_eq!(
            pl,
            "#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:4\n#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-PLAYLIST-TYPE:VOD\n\
             #EXTINF:4.000000,\nseg/0.ts\n#EXTINF:4.000000,\nseg/1.ts\n#EXT-X-ENDLIST\n"
        );
    }

    #[test]
    fn playlist_with_remainder() {
        let pl = hls_playlist(10.5);
        assert_eq!(segment_count(10.5), 3);
        assert_eq!(extinfs(&pl), ["#EXTINF:4.000000,", "#EXTINF:4.000000,", "#EXTINF:2.500000,"]);
        assert!(pl.contains("seg/2.ts\n#EXT-X-ENDLIST\n"));
        assert!(!pl.contains("seg/3.ts"));
    }

    #[test]
    fn playlist_shorter_than_one_segment() {
        assert_eq!(segment_count(1.25), 1);
        assert_eq!(extinfs(&hls_playlist(1.25)), ["#EXTINF:1.250000,"]);
    }

    #[test]
    fn playlist_zero_or_invalid_duration_has_one_segment() {
        for d in [0.0, -3.0, f64::NAN, f64::INFINITY] {
            assert_eq!(segment_count(d), 1);
            let pl = hls_playlist(d);
            assert_eq!(extinfs(&pl).len(), 1);
            assert!(pl.ends_with("#EXT-X-ENDLIST\n"));
        }
    }

    #[test]
    fn playlist_tiny_duration() {
        assert_eq!(segment_count(0.0004), 1);
        assert_eq!(extinfs(&hls_playlist(0.0004)), ["#EXTINF:0.001000,"]);
    }

    #[test]
    fn float_noise_does_not_add_a_segment() {
        assert_eq!(segment_count(8.0000001), 2);
        assert_eq!(segment_count(8.01), 3);
        assert!((segment_duration(8.01, 2) - 0.01).abs() < 1e-9);
    }

    #[test]
    fn three_hour_file() {
        let d = 3.0 * 3600.0 + 0.7;
        assert_eq!(segment_count(d), 2701);
        let pl = hls_playlist(d);
        assert_eq!(extinfs(&pl).len(), 2701);
        assert!(pl.contains("seg/2700.ts"));
        assert!((segment_duration(d, 2700) - 0.7).abs() < 1e-6);
        let total: f64 = (0..2701).map(|n| segment_duration(d, n)).sum();
        assert!((total - d).abs() < 1e-6);
    }

    #[test]
    fn segment_time_mapping() {
        assert_eq!(segment_start_sec(3), 12.0);
        assert_eq!(segment_for_time(10.5, 0.0), 0);
        assert_eq!(segment_for_time(10.5, 3.99), 0);
        assert_eq!(segment_for_time(10.5, 4.0), 1);
        assert_eq!(segment_for_time(10.5, 10.4), 2);
        assert_eq!(segment_for_time(10.5, 99.0), 2);
        assert_eq!(segment_for_time(10.5, -1.0), 0);
    }

    #[test]
    fn segment_file_names() {
        assert_eq!(segment_file_name(0), "seg_0.ts");
        assert_eq!(segment_file_name(42), "seg_42.ts");
    }

    // ---- HLS job args

    fn params(encoder: PreviewEncoder) -> HlsJobParams {
        HlsJobParams {
            input_path: "C:/v/in.mkv".into(),
            start_segment: 5,
            encoder,
            max_height: 720,
            has_audio: true,
            hwaccel: true,
            out_dir: "C:/tmp/videocrop/abc".into(),
        }
    }

    fn value_after(a: &[String], flag: &str) -> Option<String> {
        a.iter().position(|x| x == flag).and_then(|i| a.get(i + 1).cloned())
    }

    fn has(a: &[String], s: &str) -> bool {
        a.iter().any(|x| x == s)
    }

    #[test]
    fn hls_args_full_x264() {
        let a = hls_job_args(&params(PreviewEncoder::X264));
        let seg = join("C:/tmp/videocrop/abc", "seg_%d.ts");
        let pl = join("C:/tmp/videocrop/abc", "job.m3u8");
        let expected: Vec<String> = [
            "-hide_banner", "-nostats", "-loglevel", "error",
            "-hwaccel", "auto",
            "-ss", "20", "-i", "C:/v/in.mkv",
            "-map", "0:v:0", "-map", "0:a:0",
            "-vf", "scale=-2:trunc(min(ih\\,720)/2)*2,format=yuv420p",
            "-c:v", "libx264", "-preset", "ultrafast", "-tune", "zerolatency", "-crf", "26", "-sc_threshold", "0",
            "-g", "600", "-force_key_frames", "expr:gte(t,n_forced*4)",
            "-c:a", "aac", "-ac", "2", "-b:a", "160k",
            "-f", "hls", "-hls_time", "4", "-hls_segment_type", "mpegts",
            "-hls_flags", "temp_file+independent_segments", "-hls_list_size", "0",
            "-start_number", "5", "-output_ts_offset", "20",
            "-hls_segment_filename", &seg, &pl,
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(a, expected);
        assert!(seg.ends_with("seg_%d.ts") && seg.starts_with("C:/tmp/videocrop/abc"));
    }

    #[test]
    fn hls_args_per_encoder() {
        let n = hls_job_args(&params(PreviewEncoder::Nvenc));
        assert_eq!(value_after(&n, "-c:v").as_deref(), Some("h264_nvenc"));
        assert_eq!(value_after(&n, "-preset").as_deref(), Some("p1"));
        assert_eq!(value_after(&n, "-tune").as_deref(), Some("ll"));
        assert_eq!(value_after(&n, "-rc").as_deref(), Some("vbr"));
        assert_eq!(value_after(&n, "-cq").as_deref(), Some("26"));
        assert!(!has(&n, "-sc_threshold"));

        let q = hls_job_args(&params(PreviewEncoder::Qsv));
        assert_eq!(value_after(&q, "-c:v").as_deref(), Some("h264_qsv"));
        assert_eq!(value_after(&q, "-preset").as_deref(), Some("veryfast"));
        assert_eq!(value_after(&q, "-global_quality").as_deref(), Some("26"));
        assert!(!has(&q, "-crf"));

        let m = hls_job_args(&params(PreviewEncoder::Amf));
        assert_eq!(value_after(&m, "-c:v").as_deref(), Some("h264_amf"));
        assert_eq!(value_after(&m, "-quality").as_deref(), Some("speed"));
        assert_eq!(value_after(&m, "-rc").as_deref(), Some("cqp"));
        assert_eq!(value_after(&m, "-qp_i").as_deref(), Some("24"));
        assert_eq!(value_after(&m, "-qp_p").as_deref(), Some("26"));

        for a in [&n, &q, &m] {
            assert_eq!(value_after(a, "-force_key_frames").as_deref(), Some("expr:gte(t,n_forced*4)"));
            assert_eq!(value_after(a, "-hls_time").as_deref(), Some("4"));
        }
    }

    #[test]
    fn hls_args_without_audio_or_hwaccel() {
        let p = HlsJobParams { has_audio: false, hwaccel: false, start_segment: 0, max_height: 1080, ..params(PreviewEncoder::Nvenc) };
        let a = hls_job_args(&p);
        assert!(!has(&a, "-hwaccel"));
        assert!(!has(&a, "0:a:0"));
        assert!(!has(&a, "-c:a"));
        assert_eq!(value_after(&a, "-ss").as_deref(), Some("0"));
        assert_eq!(value_after(&a, "-start_number").as_deref(), Some("0"));
        assert_eq!(value_after(&a, "-output_ts_offset").as_deref(), Some("0"));
        assert_eq!(value_after(&a, "-vf").as_deref(), Some("scale=-2:trunc(min(ih\\,1080)/2)*2,format=yuv420p"));
        // -hide_banner first, the playlist last
        assert_eq!(a[0], "-hide_banner");
        assert!(a.last().unwrap().ends_with("job.m3u8"));
    }

    #[test]
    fn hls_seek_precedes_input() {
        let a = hls_job_args(&params(PreviewEncoder::X264));
        let ss = a.iter().position(|x| x == "-ss").unwrap();
        let i = a.iter().position(|x| x == "-i").unwrap();
        let hw = a.iter().position(|x| x == "-hwaccel").unwrap();
        assert!(hw < ss && ss < i);
    }

    #[test]
    fn encoder_names_roundtrip() {
        for e in PreviewEncoder::ALL {
            assert_eq!(PreviewEncoder::from_ffmpeg_name(e.ffmpeg_name()), Some(e));
        }
        assert_eq!(PreviewEncoder::from_ffmpeg_name("libx265"), None);
        assert!(!PreviewEncoder::X264.is_hardware());
        assert!(PreviewEncoder::Qsv.is_hardware());
    }

    #[test]
    fn encoder_probe() {
        assert_eq!(
            encoder_probe_args(PreviewEncoder::Amf),
            ["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", "color=c=black:s=256x144:d=0.2", "-c:v", "h264_amf", "-f", "null", "-"]
        );
    }

    // ---- frame args

    #[test]
    fn frame_args_fast_scaled() {
        assert_eq!(
            frame_args("C:/a.mp4", 12.3456, Some(640), true),
            [
                "-hide_banner", "-loglevel", "error", "-noaccurate_seek", "-skip_frame", "nokey",
                "-ss", "12.346", "-i", "C:/a.mp4", "-frames:v", "1", "-vf", "scale=640:-2",
                "-f", "image2pipe", "-c:v", "mjpeg", "-q:v", "3", "pipe:1",
            ]
        );
    }

    #[test]
    fn frame_args_exact_native() {
        assert_eq!(
            frame_args("C:/a.mp4", 5.0, None, false),
            [
                "-hide_banner", "-loglevel", "error", "-ss", "5", "-i", "C:/a.mp4", "-frames:v", "1",
                "-f", "image2pipe", "-c:v", "mjpeg", "-q:v", "3", "pipe:1",
            ]
        );
    }

    #[test]
    fn frame_args_clamps_bad_time_and_zero_width() {
        let a = frame_args("C:/a.mp4", -2.0, Some(0), false);
        assert_eq!(value_after(&a, "-ss").as_deref(), Some("0"));
        assert!(!has(&a, "-vf"));
        let a = frame_args("C:/a.mp4", f64::NAN, Some(160), true);
        assert_eq!(value_after(&a, "-ss").as_deref(), Some("0"));
        assert_eq!(value_after(&a, "-vf").as_deref(), Some("scale=160:-2"));
    }

    // ---- direct-play heuristic

    fn info(path: &str, fmt: &str, codec: &str, profile: Option<&str>, pix: Option<&str>, audio: &[&str]) -> VideoInfo {
        VideoInfo {
            path: path.into(),
            duration_sec: 10.0,
            width: 1920,
            height: 1080,
            sar_num: 1,
            sar_den: 1,
            video_codec: codec.into(),
            profile: profile.map(str::to_string),
            pix_fmt: pix.map(str::to_string),
            fps: 30.0,
            bitrate_bps: None,
            audio_streams: audio
                .iter()
                .enumerate()
                .map(|(i, c)| AudioStreamInfo { index: i as u32 + 1, codec: c.to_string(), channels: Some(2), sample_rate: None, bitrate_bps: None, language: None })
                .collect(),
            container_format: fmt.into(),
            file_size_bytes: None,
        }
    }

    const MP4: &str = "mov,mp4,m4a,3gp,3g2,mj2";
    const MKV: &str = "matroska,webm";

    #[test]
    fn direct_play_matrix() {
        let cases: Vec<(VideoInfo, bool)> = vec![
            (info("a.mp4", MP4, "h264", Some("High"), Some("yuv420p"), &["aac"]), true),
            (info("a.mp4", MP4, "h264", None, None, &[]), true),
            (info("a.mov", MP4, "h264", Some("Main"), Some("yuvj420p"), &["mp3"]), true),
            (info("a.mp4", MP4, "h264", Some("High 10"), Some("yuv420p10le"), &["aac"]), false),
            (info("a.mp4", MP4, "h264", Some("High 10"), Some("yuv420p"), &["aac"]), false),
            (info("a.mp4", MP4, "h264", Some("High 4:4:4 Predictive"), Some("yuv444p"), &["aac"]), false),
            (info("a.mp4", MP4, "h264", Some("High 4:2:2"), Some("yuv422p"), &["aac"]), false),
            (info("a.mp4", MP4, "h264", Some("High"), Some("yuv444p"), &["aac"]), false),
            (info("a.mp4", MP4, "hevc", Some("Main"), Some("yuv420p"), &["aac"]), false),
            (info("a.mp4", MP4, "av1", Some("Main"), Some("yuv420p10le"), &["opus"]), true),
            (info("a.mp4", MP4, "vp9", None, None, &["flac"]), true),
            (info("a.webm", MKV, "vp9", None, None, &["opus"]), true),
            (info("a.WEBM", MKV, "vp8", None, None, &["vorbis"]), true),
            (info("a.mkv", MKV, "vp9", None, None, &["opus"]), false),
            (info("a.mkv", MKV, "h264", Some("High"), Some("yuv420p"), &["aac"]), false),
            (info("a.avi", "avi", "h264", Some("High"), Some("yuv420p"), &["mp3"]), false),
            (info("a.ts", "mpegts", "h264", Some("High"), Some("yuv420p"), &["aac"]), false),
            (info("a.mp4", MP4, "h264", Some("High"), Some("yuv420p"), &["ac3"]), false),
            (info("a.mp4", MP4, "h264", Some("High"), Some("yuv420p"), &["pcm_s16le"]), false),
            // only the first audio stream matters
            (info("a.mp4", MP4, "h264", Some("High"), Some("yuv420p"), &["aac", "ac3"]), true),
            (info("a.mp4", MP4, "h264", Some("High"), Some("yuv420p"), &["eac3", "aac"]), false),
            (info("a.mp4", MP4, "mpeg4", None, None, &["aac"]), false),
            (info("a.mp4", MP4, "prores", None, None, &[]), false),
        ];
        for (i, (v, want)) in cases.iter().enumerate() {
            assert_eq!(direct_likely_playable(v), *want, "case {i}: {} {} {:?} {:?} {:?}", v.path, v.video_codec, v.profile, v.pix_fmt, v.audio_streams.first().map(|a| &a.codec));
        }
    }
}
