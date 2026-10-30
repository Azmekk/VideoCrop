//! Picks the preview (HLS) encoder once: tries each `PreviewEncoder::ALL` with a tiny synthetic
//! encode, first exit-0 wins. The winner is cached in settings so later launches skip probing.
//! Until detection finishes, sessions use libx264.

use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::core::preview::{encoder_probe_args, PreviewEncoder};
use crate::media::{process, tools};
use crate::state::AppState;

const PROBE_TIMEOUT: Duration = Duration::from_secs(15);

/// Idempotent; safe to call from several places (startup, after a tools download).
pub async fn ensure(state: AppState) {
    if state.hwdetect_started.swap(true, Ordering::SeqCst) {
        return;
    }
    let cached = state.settings_snapshot().preview_encoder.as_deref().and_then(PreviewEncoder::from_ffmpeg_name);
    if let Some(enc) = cached {
        set(&state, enc);
        return;
    }
    let Ok(t) = tools::require(&state) else {
        // Tools missing: allow a later call (after download) to run detection.
        state.hwdetect_started.store(false, Ordering::SeqCst);
        return;
    };
    let mut winner = PreviewEncoder::X264;
    for enc in PreviewEncoder::ALL {
        if enc == PreviewEncoder::X264 {
            break; // always works, nothing to probe
        }
        let args = encoder_probe_args(enc);
        if let Ok(out) = process::run_capture(&t.ffmpeg, &args, Some(PROBE_TIMEOUT)).await {
            if out.status.success() {
                winner = enc;
                break;
            }
        }
    }
    set(&state, winner);
    let _ = state.update_settings(|s| s.preview_encoder = Some(winner.ffmpeg_name().to_owned()));
}

fn set(state: &AppState, enc: PreviewEncoder) {
    *state.preview_encoder.write().unwrap_or_else(|e| e.into_inner()) = Some(enc);
}
