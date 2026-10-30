//! Preview playback: on-demand HLS (fixed-length segments over a synthetic VOD playlist),
//! still-frame grabs, preview-encoder detection, and the direct-play compatibility hint.

use super::models::VideoInfo;

/// Every segment is exactly this long except the last.
pub const SEGMENT_SEC: f64 = 4.0;

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

pub fn segment_count(duration_sec: f64) -> u32 {
    let _ = duration_sec;
    todo!("core agent")
}

/// VOD playlist listing every segment as `seg/<n>.ts` (relative to the playlist URL).
pub fn hls_playlist(duration_sec: f64) -> String {
    let _ = duration_sec;
    todo!("core agent")
}

/// File name the HLS job writes for segment `n` inside `out_dir`.
pub fn segment_file_name(n: u32) -> String {
    format!("seg_{n}.ts")
}

pub fn hls_job_args(p: &HlsJobParams) -> Vec<String> {
    let _ = p;
    todo!("core agent")
}

/// Tiny synthetic encode (`-f lavfi color ... -f null -`) used to test whether an encoder works.
pub fn encoder_probe_args(encoder: PreviewEncoder) -> Vec<String> {
    let _ = encoder;
    todo!("core agent")
}

/// Single JPEG to stdout. `fast` = keyframe-snapped (`-noaccurate_seek`) for live scrubbing.
/// `width` None = native resolution (used by the crop editor).
pub fn frame_args(input_path: &str, time_sec: f64, width: Option<u32>, fast: bool) -> Vec<String> {
    let _ = (input_path, time_sec, width, fast);
    todo!("core agent")
}

/// Heuristic for whether WebView2 can play the original file directly.
pub fn direct_likely_playable(info: &VideoInfo) -> bool {
    let _ = info;
    todo!("core agent")
}
