//! One open video: id, probe info, temp dir, HLS job state (see `stream`) and frame grabs.

use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::{watch, Semaphore};

use crate::core::models::VideoInfo;
use crate::core::preview::{self, HlsJobParams, PreviewEncoder};
use crate::media::process;
use crate::media::stream::{self, HlsState};
use crate::state::{lock, SharedEncoder};

const LOCK_FILE: &str = ".lock";
const FRAME_TIMEOUT: Duration = Duration::from_secs(20);
/// Filmstrip thumbnails (w <= this) run up to `FILMSTRIP_CONCURRENCY` at once.
pub const FILMSTRIP_MAX_WIDTH: u32 = 200;
const FILMSTRIP_CONCURRENCY: usize = 2;

/// The core functions a session uses. Production always uses `PreviewFns::default()` (core);
/// the seam only exists so the runtime can be exercised before/independently of core.
#[derive(Clone, Copy)]
pub struct PreviewFns {
    pub hls_job_args: fn(&HlsJobParams) -> Vec<String>,
    pub frame_args: fn(&str, f64, Option<u32>, bool) -> Vec<String>,
    pub segment_count: fn(f64) -> u32,
    pub hls_playlist: fn(f64) -> String,
}

impl Default for PreviewFns {
    fn default() -> Self {
        Self {
            hls_job_args: preview::hls_job_args,
            frame_args: preview::frame_args,
            segment_count: preview::segment_count,
            hls_playlist: preview::hls_playlist,
        }
    }
}

pub struct Session {
    pub id: String,
    pub info: VideoInfo,
    pub dir: PathBuf,
    pub ffmpeg: PathBuf,
    pub fns: PreviewFns,
    pub(crate) hls: Mutex<HlsState>,
    encoder: SharedEncoder,
    closed: AtomicBool,
    scrub_gen: watch::Sender<u64>,
    filmstrip: Semaphore,
    /// Held open without sharing so the startup sweep of another instance skips this dir.
    lock_file: Mutex<Option<fs::File>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FrameError {
    /// A newer scrub request replaced this one.
    Superseded,
    Closed,
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    /// Live scrubbing: only the newest request matters.
    Scrub,
    /// Timeline thumbnails: bounded concurrency.
    Filmstrip,
    /// Crop editor: exact frame, native size.
    Exact,
}

pub fn classify_frame(width: Option<u32>, fast: bool) -> FrameKind {
    match (fast, width) {
        (false, _) => FrameKind::Exact,
        (true, Some(w)) if w <= FILMSTRIP_MAX_WIDTH => FrameKind::Filmstrip,
        (true, _) => FrameKind::Scrub,
    }
}

pub fn new_session_id() -> String {
    format!("{:016x}", rand::random::<u64>())
}

impl Session {
    /// Creates the temp dir and starts the HLS supervisor tick. Must run inside a tokio runtime.
    pub fn create(
        info: VideoInfo,
        ffmpeg: PathBuf,
        temp_root: &Path,
        encoder: SharedEncoder,
        fns: PreviewFns,
    ) -> Result<Arc<Self>, String> {
        let id = new_session_id();
        let dir = temp_root.join(&id);
        fs::create_dir_all(&dir).map_err(|e| format!("Could not create temp folder: {e}"))?;
        let lock_file = open_exclusive(&dir.join(LOCK_FILE)).ok();
        let session = Arc::new(Self {
            id,
            info,
            dir,
            ffmpeg,
            fns,
            hls: Mutex::new(HlsState::default()),
            encoder,
            closed: AtomicBool::new(false),
            scrub_gen: watch::channel(0).0,
            filmstrip: Semaphore::new(FILMSTRIP_CONCURRENCY),
            lock_file: Mutex::new(lock_file),
        });
        stream::spawn_supervisor(Arc::downgrade(&session));
        Ok(session)
    }

    pub fn input(&self) -> &str {
        &self.info.path
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    /// Encoder for new HLS jobs: hwdetect's pick, libx264 until detection finished.
    pub fn preferred_encoder(&self) -> PreviewEncoder {
        self.encoder.read().unwrap_or_else(|e| e.into_inner()).unwrap_or(PreviewEncoder::X264)
    }

    /// Stops the HLS job (waiting briefly so its files unlock). The temp dir is removed when
    /// the last reference drops (an encode may still be using it for two-pass stats).
    pub async fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.scrub_gen.send_modify(|g| *g += 1);
        let job = lock(&self.hls).take_job();
        if let Some(mut child) = job {
            let _ = child.start_kill();
            let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
        }
    }

    pub async fn grab_frame(&self, time_sec: f64, width: Option<u32>, fast: bool) -> Result<Vec<u8>, FrameError> {
        if self.is_closed() {
            return Err(FrameError::Closed);
        }
        let t = time_sec.clamp(0.0, self.info.duration_sec.max(0.0));
        let args = (self.fns.frame_args)(self.input(), t, width, fast);
        let run = async {
            let out = process::run_capture(&self.ffmpeg, &args, Some(FRAME_TIMEOUT))
                .await
                .map_err(|e| FrameError::Failed(format!("ffmpeg: {e}")))?;
            if out.status.success() && !out.stdout.is_empty() {
                Ok(out.stdout)
            } else {
                Err(FrameError::Failed(out.stderr))
            }
        };
        match classify_frame(width, fast) {
            FrameKind::Scrub => coalesce(&self.scrub_gen, run).await.unwrap_or(Err(FrameError::Superseded)),
            FrameKind::Filmstrip => {
                let _permit = self.filmstrip.acquire().await.map_err(|_| FrameError::Closed)?;
                run.await
            }
            FrameKind::Exact => run.await,
        }
    }
}

/// Runs `work` unless a newer caller bumps `gen` first; then `work` is dropped (which kills
/// its ffmpeg via kill_on_drop) and None is returned.
pub async fn coalesce<T>(gen: &watch::Sender<u64>, work: impl Future<Output = T>) -> Option<T> {
    let mut mine = 0;
    gen.send_modify(|g| {
        *g += 1;
        mine = *g;
    });
    let mut rx = gen.subscribe();
    let superseded = async move {
        loop {
            if *rx.borrow_and_update() != mine {
                return;
            }
            if rx.changed().await.is_err() {
                std::future::pending::<()>().await;
            }
        }
    };
    tokio::select! {
        r = work => Some(r),
        _ = superseded => None,
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // Release our lock first, then delete. Failures are left for the startup sweep.
        drop(lock(&self.lock_file).take());
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[cfg(windows)]
fn open_exclusive(path: &Path) -> std::io::Result<fs::File> {
    use std::os::windows::fs::OpenOptionsExt;
    fs::OpenOptions::new().write(true).create(true).truncate(true).share_mode(0).open(path)
}

#[cfg(not(windows))]
fn open_exclusive(path: &Path) -> std::io::Result<fs::File> {
    fs::OpenOptions::new().write(true).create(true).truncate(true).open(path)
}

/// Deletes session dirs left behind by crashed runs. Dirs whose lock file is still held by
/// another running instance are skipped. Blocking; run on the blocking pool.
pub fn sweep_stale(temp_root: &Path) {
    let Ok(entries) = fs::read_dir(temp_root) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let lock = path.join(LOCK_FILE);
        match fs::remove_file(&lock) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => continue, // in use by a live instance
        }
        let _ = fs::remove_dir_all(&path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_classification() {
        assert_eq!(classify_frame(Some(640), true), FrameKind::Scrub);
        assert_eq!(classify_frame(None, true), FrameKind::Scrub);
        assert_eq!(classify_frame(Some(160), true), FrameKind::Filmstrip);
        assert_eq!(classify_frame(Some(200), true), FrameKind::Filmstrip);
        assert_eq!(classify_frame(Some(160), false), FrameKind::Exact);
        assert_eq!(classify_frame(None, false), FrameKind::Exact);
    }

    #[tokio::test]
    async fn coalesce_newer_request_supersedes_older() {
        let gen = Arc::new(watch::channel(0u64).0);
        let g1 = gen.clone();
        let first = tokio::spawn(async move {
            coalesce(&g1, async {
                tokio::time::sleep(Duration::from_secs(5)).await;
                1
            })
            .await
        });
        tokio::time::sleep(Duration::from_millis(50)).await;
        let second = coalesce(&gen, async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            2
        })
        .await;
        assert_eq!(second, Some(2));
        let first = tokio::time::timeout(Duration::from_secs(1), first).await.unwrap().unwrap();
        assert_eq!(first, None);
    }

    #[tokio::test]
    async fn coalesce_alone_completes() {
        let gen = watch::channel(0u64).0;
        assert_eq!(coalesce(&gen, async { 7 }).await, Some(7));
    }

    #[test]
    fn sweep_removes_unlocked_and_keeps_locked() {
        let root = std::env::temp_dir().join(format!("vc-sweep-test-{}", std::process::id()));
        let stale = root.join("stale");
        let live = root.join("live");
        fs::create_dir_all(&stale).unwrap();
        fs::create_dir_all(&live).unwrap();
        fs::write(stale.join(LOCK_FILE), b"").unwrap();
        fs::write(stale.join("seg_0.ts"), b"x").unwrap();
        let held = open_exclusive(&live.join(LOCK_FILE)).unwrap();
        sweep_stale(&root);
        assert!(!stale.exists());
        assert!(live.exists());
        drop(held);
        sweep_stale(&root);
        assert!(!live.exists());
        let _ = fs::remove_dir_all(&root);
    }
}
