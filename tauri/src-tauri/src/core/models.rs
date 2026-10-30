//! Data contract shared by the backend and the frontend (mirrored in `src/lib/types.ts`).
//! Everything crosses IPC as camelCase JSON. Times are seconds as f64, sizes are bytes.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------- probe

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    pub path: String,
    pub duration_sec: f64,
    pub width: u32,
    pub height: u32,
    pub sar_num: u32,
    pub sar_den: u32,
    /// ffprobe `codec_name` of the first video stream, e.g. "h264", "hevc", "av1".
    pub video_codec: String,
    /// ffprobe `profile`, e.g. "High", "Main 10".
    pub profile: Option<String>,
    /// ffprobe `pix_fmt`, e.g. "yuv420p", "yuv420p10le".
    pub pix_fmt: Option<String>,
    pub fps: f64,
    pub bitrate_bps: Option<u64>,
    pub audio_streams: Vec<AudioStreamInfo>,
    /// ffprobe `format_name`, e.g. "mov,mp4,m4a,3gp,3g2,mj2" or "matroska,webm".
    pub container_format: String,
    pub file_size_bytes: Option<u64>,
}

impl VideoInfo {
    /// Display aspect ratio accounting for anamorphic SAR.
    pub fn display_aspect_ratio(&self) -> f64 {
        if self.height == 0 {
            return 1.0;
        }
        let n = if self.sar_num > 0 { self.sar_num } else { 1 } as f64;
        let d = if self.sar_den > 0 { self.sar_den } else { 1 } as f64;
        self.width as f64 * n / (self.height as f64 * d)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioStreamInfo {
    pub index: u32,
    pub codec: String,
    pub channels: Option<u32>,
    pub sample_rate: Option<u32>,
    pub bitrate_bps: Option<u64>,
    pub language: Option<String>,
}

// ---------------------------------------------------------------- edit specs

/// Temporal trim. `accurate` = decode-then-cut (`-ss` after `-i`); otherwise keyframe-snapped.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CutSpec {
    pub start_sec: f64,
    pub end_sec: f64,
    pub accurate: bool,
}

/// Always in SOURCE-pixel coordinates, independent of resize.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CropSpec {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Applied to the POST-crop frame. When `aspect_locked`, height is derived by ffmpeg (`-2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResizeSpec {
    pub width: u32,
    pub height: u32,
    pub aspect_locked: bool,
}

// ---------------------------------------------------------------- compression

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoCodec {
    H264,
    H265,
    Av1,
    Vp9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioCodec {
    Aac,
    Opus,
    Mp3,
    Copy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RateControl {
    Crf,
    Cbr,
    Vbr,
    /// Two-pass, bitrate computed from `target_size_bytes` and the (cut) duration.
    TargetSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpeedPreset {
    Ultrafast,
    Superfast,
    Veryfast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    Veryslow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PixelFormat {
    Yuv420p,
    Yuv420p10le,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressionSpec {
    pub codec: VideoCodec,
    pub rate_control: RateControl,
    pub crf: u32,
    pub target_bitrate_bps: Option<u64>,
    pub target_size_bytes: Option<u64>,
    pub speed: SpeedPreset,
    pub audio_codec: AudioCodec,
    pub audio_bitrate_kbps: u32,
    pub pixel_format: PixelFormat,
    /// Container extension without dot ("mp4", "mkv", "webm"); None = codec default.
    pub container: Option<String>,
}

/// What the user picked in the Compression section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CompressionChoice {
    /// Re-encoding off: stream copy. Crop/resize are ignored.
    Off,
    /// One of the quality presets (`PresetLibrary`), by id.
    #[serde(rename_all = "camelCase")]
    Quality { preset_id: String },
    /// A share-target preset (Discord, YouTube, ...), by id.
    #[serde(rename_all = "camelCase")]
    Platform { platform_id: String },
    /// Advanced panel.
    Custom { spec: CompressionSpec },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputMode {
    Everything,
    AudioOnly,
    VideoOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PresetCategory {
    Compatibility,
    Optimization,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityPreset {
    pub id: String,
    pub name: String,
    pub category: PresetCategory,
    pub description: String,
    pub warning: Option<String>,
    pub spec: CompressionSpec,
}

/// Share-target preset. The planner turns this into a concrete CompressionSpec + resize cap
/// for the current source and cut, e.g. picking a resolution that still looks good at 10 MB.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformPreset {
    pub id: String,
    pub name: String,
    /// Icon key the frontend maps to an SVG ("discord", "youtube", "x", "instagram", ...).
    pub icon: String,
    pub description: String,
    /// Hard upload cap; when set the encode is two-pass target-size.
    pub max_size_bytes: Option<u64>,
    /// Output must fit inside this box (aspect preserved, never upscaled).
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    /// Base spec (codec/audio/speed; CRF used when there is no size cap).
    pub spec: CompressionSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetCatalog {
    pub quality: Vec<QualityPreset>,
    pub platforms: Vec<PlatformPreset>,
    pub default_quality_id: String,
}

// ---------------------------------------------------------------- planning

/// Everything the UI has configured. Sent to `plan_job` on change and to `encode_start`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobDraft {
    pub cut: Option<CutSpec>,
    pub crop: Option<CropSpec>,
    pub resize: Option<ResizeSpec>,
    pub compression: CompressionChoice,
    pub mode: OutputMode,
    /// None = next to the source file.
    pub output_dir: Option<String>,
}

/// The resolved, ready-to-run job. `summary` is the single source of truth shown before Process.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobPlan {
    pub output_path: String,
    pub summary: String,
    /// Final frame size (None for audio-only).
    pub output_width: Option<u32>,
    pub output_height: Option<u32>,
    /// Resolved encoder spec; None = stream copy.
    pub spec: Option<CompressionSpec>,
    pub crop: Option<CropSpec>,
    /// Effective resize after platform caps (may differ from the draft's).
    pub resize: Option<ResizeSpec>,
    pub cut: Option<CutSpec>,
    pub mode: OutputMode,
    pub two_pass: bool,
    pub estimated_size_bytes: Option<u64>,
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------- runtime events

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodeProgress {
    pub frame: u64,
    pub fps: f64,
    pub out_time_sec: f64,
    pub speed: f64,
    pub finished: bool,
}

/// Emitted as `encode://progress`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodeStatus {
    /// 0..1 across all passes.
    pub fraction: f64,
    /// 1-based; 1 of 1 for single-pass.
    pub pass: u32,
    pub pass_count: u32,
    pub fps: f64,
    pub speed: f64,
    pub eta_sec: Option<f64>,
}

/// Emitted as `encode://done`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodeResult {
    pub ok: bool,
    pub cancelled: bool,
    pub output_path: String,
    pub output_size_bytes: Option<u64>,
    pub error: Option<String>,
    /// Last ~4 KB of ffmpeg stderr on failure.
    pub stderr_tail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    pub ffmpeg: Option<String>,
    pub ffprobe: Option<String>,
    pub ready: bool,
    pub tools_dir: String,
    /// Where the tools came from: "bundled" (downloaded by VideoCrop), "path" or "dev".
    pub source: Option<String>,
    /// Why the ffmpeg found on PATH can't be used (shown on the setup screen), if one was found.
    pub problem: Option<String>,
}

/// Emitted as `tools://progress`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDownloadProgress {
    pub fraction: f64,
    pub status: String,
}

/// Returned by `open_video`. URLs are served by the `stream` protocol for this session only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedVideo {
    pub info: VideoInfo,
    pub session: String,
    /// Original file with HTTP range support.
    pub direct_url: String,
    /// On-demand HLS playlist (transcoded/remuxed preview).
    pub hls_url: String,
    /// Base for still frames: `{frame_url}?t=<sec>&w=<px>&fast=<0|1>` -> image/jpeg.
    pub frame_url: String,
    /// Hint from the backend: true when the container/codecs are ones WebView2 can usually
    /// play. The frontend still confirms with `canPlayType` and falls back on `error`.
    pub direct_likely_playable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// "system" | "light" | "dark"
    pub theme: String,
    pub last_output_dir: Option<String>,
    pub default_compression: Option<CompressionChoice>,
    pub recent_files: Vec<String>,
    /// Cached result of preview-encoder detection ("h264_nvenc", "libx264", ...).
    pub preview_encoder: Option<String>,
    /// Release version the user chose "Skip this version" for (no leading "v").
    pub skipped_update: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            last_output_dir: None,
            default_compression: None,
            recent_files: Vec::new(),
            preview_encoder: None,
            skipped_update: None,
        }
    }
}
