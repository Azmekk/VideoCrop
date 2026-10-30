//! Runs a `JobPlan`'s ffmpeg passes, emits `encode://progress` / `encode://done`, cancels.
//! Only one encode runs at a time.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::watch;

use crate::core::encode::build_passes;
use crate::core::models::{EncodeResult, EncodeStatus, JobPlan};
use crate::core::progress::ProgressParser;
use crate::media::process::{self, Pipe};
use crate::media::session::Session;
use crate::state::{lock, AppState};

const EMIT_INTERVAL: Duration = Duration::from_millis(100);
const STATS_NAME: &str = "passlog";

pub struct EncodeHandle {
    id: u64,
    cancel: watch::Sender<bool>,
    done: watch::Receiver<bool>,
}

impl EncodeHandle {
    pub fn cancel(&self) {
        let _ = self.cancel.send(true);
    }

    /// Resolves when the encode task has finished (including partial-output cleanup).
    pub fn finished(&self) -> impl std::future::Future<Output = ()> + Send + 'static {
        let mut rx = self.done.clone();
        async move {
            let _ = rx.wait_for(|d| *d).await;
        }
    }
}

/// Overall fraction (0..1) and ETA given progress inside pass `pass_idx` (0-based).
pub fn overall(pass_idx: u32, pass_count: u32, out_time_sec: f64, duration_sec: f64, speed: f64) -> (f64, Option<f64>) {
    let dur = duration_sec.max(0.001);
    let in_pass = (out_time_sec / dur).clamp(0.0, 1.0);
    let count = pass_count.max(1) as f64;
    let fraction = ((pass_idx as f64 + in_pass) / count).clamp(0.0, 1.0);
    let remaining_media = (count - pass_idx as f64 - in_pass).max(0.0) * dur;
    let eta = (speed > 0.0 && speed.is_finite()).then(|| remaining_media / speed);
    (fraction, eta)
}

/// Builds the passes, then runs them in the background. Errors here are synchronous
/// (busy, bad plan); runtime failures arrive as `encode://done` with `ok: false`.
pub fn start(app: AppHandle, state: AppState, session: Arc<Session>, ffmpeg: PathBuf, plan: JobPlan) -> Result<(), String> {
    let mut slot = lock(&state.encode);
    if slot.is_some() {
        return Err("An encode is already running.".into());
    }
    let output = PathBuf::from(&plan.output_path);
    if let Some(dir) = output.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("Could not create output folder: {e}"))?;
    }
    let stats_prefix = session.dir.join(STATS_NAME);
    let passes = build_passes(&plan, session.input(), &stats_prefix.to_string_lossy())?;
    if passes.is_empty() {
        return Err("Nothing to run.".into());
    }
    let duration = plan.cut.map(|c| c.end_sec - c.start_sec).unwrap_or(session.info.duration_sec);

    let (cancel_tx, cancel_rx) = watch::channel(false);
    let (done_tx, done_rx) = watch::channel(false);
    let id = rand::random::<u64>();
    *slot = Some(EncodeHandle { id, cancel: cancel_tx, done: done_rx });
    drop(slot);

    let task_state = state.clone();
    tauri::async_runtime::spawn(async move {
        let result = run(&app, &ffmpeg, &passes, duration, &output, cancel_rx).await;
        remove_stats(&session.dir);
        drop(session);
        {
            let mut slot = lock(&task_state.encode);
            if slot.as_ref().is_some_and(|h| h.id == id) {
                *slot = None;
            }
        }
        let _ = app.emit("encode://done", result);
        let _ = done_tx.send(true);
    });
    Ok(())
}

async fn run(
    app: &AppHandle,
    ffmpeg: &Path,
    passes: &[Vec<String>],
    duration: f64,
    output: &Path,
    mut cancel: watch::Receiver<bool>,
) -> EncodeResult {
    let pass_count = passes.len() as u32;
    let output_path = output.to_string_lossy().into_owned();
    let fail = |error: String, stderr_tail: Option<String>, cancelled: bool| {
        let _ = std::fs::remove_file(output);
        EncodeResult { ok: false, cancelled, output_path: output_path.clone(), output_size_bytes: None, error: Some(error), stderr_tail }
    };

    for (idx, args) in passes.iter().enumerate() {
        let idx = idx as u32;
        let mut child = match process::spawn(ffmpeg, args, Pipe::Piped, Pipe::Piped) {
            Ok(c) => c,
            Err(e) => return fail(format!("Could not start ffmpeg: {e}"), None, false),
        };
        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        let stderr_task = tokio::spawn(process::tail_lines(stderr, process::STDERR_TAIL));

        let progress = async {
            let mut parser = ProgressParser::new();
            let mut lines = BufReader::new(stdout).lines();
            let mut last_emit: Option<Instant> = None;
            while let Ok(Some(line)) = lines.next_line().await {
                let Some(p) = parser.on_line(&line) else { continue };
                if !p.finished && last_emit.is_some_and(|t| t.elapsed() < EMIT_INTERVAL) {
                    continue;
                }
                last_emit = Some(Instant::now());
                let out_time = if p.finished { duration } else { p.out_time_sec };
                let (fraction, eta_sec) = overall(idx, pass_count, out_time, duration, p.speed);
                let _ = app.emit(
                    "encode://progress",
                    EncodeStatus { fraction, pass: idx + 1, pass_count, fps: p.fps, speed: p.speed, eta_sec },
                );
            }
        };
        let exited = async {
            progress.await;
            child.wait().await
        };
        let cancelled = async {
            let _ = cancel.wait_for(|c| *c).await;
        };
        let status = tokio::select! {
            s = exited => Some(s),
            _ = cancelled => None,
        };
        let Some(status) = status else {
            let _ = child.kill().await;
            stderr_task.abort();
            return fail("Cancelled".into(), None, true);
        };
        let tail = stderr_task.await.unwrap_or_default();
        match status {
            Ok(s) if s.success() => {}
            Ok(s) => {
                let code = s.code().map(|c| c.to_string()).unwrap_or_else(|| "?".into());
                let what = if pass_count > 1 { format!(" (pass {} of {pass_count})", idx + 1) } else { String::new() };
                return fail(format!("ffmpeg failed with exit code {code}{what}"), Some(tail), false);
            }
            Err(e) => return fail(format!("ffmpeg: {e}"), Some(tail), false),
        }
    }

    let size = std::fs::metadata(output).ok().map(|m| m.len());
    EncodeResult { ok: true, cancelled: false, output_path, output_size_bytes: size, error: None, stderr_tail: None }
}

fn remove_stats(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        if e.file_name().to_string_lossy().starts_with(STATS_NAME) {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_pass_fraction_and_eta() {
        let (f, eta) = overall(0, 1, 30.0, 60.0, 2.0);
        assert!((f - 0.5).abs() < 1e-9);
        assert!((eta.unwrap() - 15.0).abs() < 1e-9);
    }

    #[test]
    fn two_pass_fraction_spans_passes() {
        let (f, eta) = overall(1, 2, 30.0, 60.0, 1.0);
        assert!((f - 0.75).abs() < 1e-9);
        assert!((eta.unwrap() - 30.0).abs() < 1e-9);
        let (f, eta) = overall(0, 2, 60.0, 60.0, 3.0);
        assert!((f - 0.5).abs() < 1e-9);
        assert!((eta.unwrap() - 20.0).abs() < 1e-9);
    }

    #[test]
    fn no_speed_no_eta_and_clamped() {
        let (f, eta) = overall(0, 1, 999.0, 60.0, 0.0);
        assert_eq!(f, 1.0);
        assert_eq!(eta, None);
    }
}
