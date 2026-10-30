//! ffmpeg argument construction for the final encode. Port of EncodeCommandBuilder.cs.
//! Filter chain order is fixed: `crop=W:H:X:Y,scale=W2:H2:flags=lanczos`.

use super::models::*;

/// Builds every ffmpeg invocation needed for `plan` (1 for single-pass, 2 for target-size).
/// `stats_prefix` is the `-passlogfile` prefix used only by two-pass.
/// Every invocation ends with `-progress pipe:1 -nostats <output>` (pass 1 writes to NUL).
pub fn build_passes(plan: &JobPlan, input_path: &str, stats_prefix: &str) -> Result<Vec<Vec<String>>, String> {
    let _ = (plan, input_path, stats_prefix);
    todo!("core agent")
}

/// `crop=...,scale=...` or None when neither is set.
pub fn video_filter(crop: Option<&CropSpec>, resize: Option<&ResizeSpec>) -> Option<String> {
    let _ = (crop, resize);
    todo!("core agent")
}

/// Video bitrate that makes `target_bytes` after reserving audio, with a safety margin (0.97).
pub fn compute_target_bitrate(target_bytes: u64, duration_sec: f64, audio_kbps: u32, safety_margin: f64) -> u64 {
    let _ = (target_bytes, duration_sec, audio_kbps, safety_margin);
    todo!("core agent")
}
