//! On-demand HLS supervisor. One continuous ffmpeg job per session writes `seg_<n>.ts` from
//! its start segment onward; segment requests serve, wait for, or restart that job.
//!
//! Rules (ARCHITECTURE.md "On-demand HLS"):
//! - file exists -> serve
//! - job running and `start <= n <= next + LOOKAHEAD` -> wait (poll 40 ms, 30 s timeout)
//! - otherwise -> kill + restart at n (only the newest request may restart; older, abandoned
//!   requests give up instead of fighting over the job)
//! - throttle: kill the job when it is more than `MAX_AHEAD` segments past the newest request
//! - a job that dies before its first segment is retried without hwaccel, then with libx264.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use tokio::process::Child;

use crate::core::preview::{segment_file_name, HlsJobParams, PreviewEncoder};
use crate::media::process::{self, Pipe, TailBuffer};
use crate::media::session::Session;
use crate::state::lock;

pub const LOOKAHEAD: u32 = 4;
pub const MAX_AHEAD: u32 = 12;
pub const POLL: Duration = Duration::from_millis(40);
pub const WAIT_TIMEOUT: Duration = Duration::from_secs(30);
const SUPERVISOR_TICK: Duration = Duration::from_millis(250);
/// Preview output height cap for every encoder; 1440p OBS clips encode far faster at 720p.
pub const MAX_HEIGHT: u32 = 720;

// ------------------------------------------------------------------ pure decision logic

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobView {
    pub start: u32,
    /// First segment not yet on disk (contiguous from `start`).
    pub next: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Serve,
    Wait,
    Restart,
    /// A newer request owns the job and this segment isn't coming; stop waiting.
    GiveUp,
}

/// `job` must describe a live job (dead jobs are cleared by `HlsState::poll` first).
pub fn decide(n: u32, exists: bool, job: Option<JobView>, is_latest: bool) -> Decision {
    if exists {
        return Decision::Serve;
    }
    if let Some(j) = job {
        if j.start <= n && n <= j.next.saturating_add(LOOKAHEAD) {
            return Decision::Wait;
        }
    }
    if is_latest {
        Decision::Restart
    } else {
        Decision::GiveUp
    }
}

pub fn should_throttle(job_next: u32, newest_request: u32) -> bool {
    job_next > newest_request.saturating_add(MAX_AHEAD)
}

/// Next configuration to try after a job died before producing anything.
/// `(hwaccel, encoder)` -> Some(new config) or None when everything was tried.
pub fn fallback(hwaccel: bool, encoder: PreviewEncoder) -> Option<(bool, PreviewEncoder)> {
    if hwaccel {
        Some((false, encoder))
    } else if encoder != PreviewEncoder::X264 {
        Some((false, PreviewEncoder::X264))
    } else {
        None
    }
}

// ------------------------------------------------------------------ runtime state

struct Job {
    child: Child,
    start: u32,
    next: u32,
    hwaccel: bool,
    encoder: PreviewEncoder,
    stderr: Arc<Mutex<TailBuffer>>,
}

#[derive(Default)]
pub struct HlsState {
    job: Option<Job>,
    req_counter: u64,
    latest_req: u64,
    newest_segment: u32,
    /// Session-wide degradations after failures.
    no_hwaccel: bool,
    force_x264: bool,
    /// Start segment of the last job that failed even with every fallback, and why.
    failed: Option<(u32, String)>,
}

impl HlsState {
    pub(crate) fn take_job(&mut self) -> Option<Child> {
        self.job.take().map(|j| j.child)
    }

    fn view(&self) -> Option<JobView> {
        self.job.as_ref().map(|j| JobView { start: j.start, next: j.next })
    }

    /// Updates progress, applies throttle, detects death and applies the retry chain.
    fn poll(&mut self, s: &Session) {
        let seg_count = (s.fns.segment_count)(s.info.duration_sec);
        let Some(job) = self.job.as_mut() else { return };
        // try_wait before scanning so files written right before exit are counted.
        let exited = !matches!(job.child.try_wait(), Ok(None));
        while job.next < seg_count && segment_path(s, job.next).is_file() {
            job.next += 1;
        }
        if !exited {
            if should_throttle(job.next, self.newest_segment) {
                self.job = None; // kill_on_drop
            }
            return;
        }
        let job = self.job.take().expect("job present");
        if job.next > job.start || job.next >= seg_count {
            return; // produced output (finished, or died mid-way: the next request restarts it)
        }
        let tail = lock(&job.stderr).as_string();
        match fallback(job.hwaccel, job.encoder) {
            Some((hwaccel, encoder)) => {
                if !hwaccel {
                    self.no_hwaccel = true;
                }
                if encoder == PreviewEncoder::X264 && job.encoder != PreviewEncoder::X264 {
                    self.force_x264 = true;
                }
                if let Err(e) = self.start_job(s, job.start) {
                    self.failed = Some((job.start, e));
                }
            }
            None => self.failed = Some((job.start, tail)),
        }
    }

    fn start_job(&mut self, s: &Session, start: u32) -> Result<(), String> {
        self.job = None;
        self.failed = None;
        let encoder = if self.force_x264 { PreviewEncoder::X264 } else { s.preferred_encoder() };
        let params = HlsJobParams {
            input_path: s.input().to_owned(),
            start_segment: start,
            encoder,
            max_height: MAX_HEIGHT,
            has_audio: !s.info.audio_streams.is_empty(),
            hwaccel: !self.no_hwaccel,
            out_dir: s.dir.to_string_lossy().into_owned(),
        };
        let args = (s.fns.hls_job_args)(&params);
        let mut child = process::spawn(&s.ffmpeg, &args, Pipe::Null, Pipe::Piped)
            .map_err(|e| format!("Could not start ffmpeg: {e}"))?;
        let stderr = Arc::new(Mutex::new(TailBuffer::new(process::STDERR_TAIL)));
        if let Some(err) = child.stderr.take() {
            let sink = stderr.clone();
            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut lines = tokio::io::BufReader::new(err).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    lock(&sink).push(&line);
                }
            });
        }
        self.job = Some(Job { child, start, next: start, hwaccel: params.hwaccel, encoder, stderr });
        Ok(())
    }
}

pub fn segment_path(s: &Session, n: u32) -> PathBuf {
    s.dir.join(segment_file_name(n))
}

#[derive(Debug, PartialEq, Eq)]
pub enum SegmentError {
    NotFound,
    /// Abandoned request (a newer one moved the job elsewhere).
    Superseded,
    Timeout,
    Failed(String),
}

/// Serves segment `n`, starting/restarting the job as needed. Never holds the lock across
/// an await.
pub async fn serve_segment(s: Arc<Session>, n: u32) -> Result<Vec<u8>, SegmentError> {
    if s.is_closed() || n >= (s.fns.segment_count)(s.info.duration_sec) {
        return Err(SegmentError::NotFound);
    }
    let my_req = {
        let mut st = lock(&s.hls);
        st.req_counter += 1;
        st.latest_req = st.req_counter;
        st.newest_segment = n;
        st.req_counter
    };
    let path = segment_path(&s, n);
    let deadline = Instant::now() + WAIT_TIMEOUT;
    loop {
        if s.is_closed() {
            return Err(SegmentError::NotFound);
        }
        let decision = {
            let mut st = lock(&s.hls);
            st.poll(&s);
            if st.job.is_none() {
                if let Some((at, why)) = &st.failed {
                    if *at == n {
                        return Err(SegmentError::Failed(why.clone()));
                    }
                }
            }
            let d = decide(n, path.is_file(), st.view(), st.latest_req == my_req);
            if d == Decision::Restart {
                st.start_job(&s, n).map_err(SegmentError::Failed)?;
            }
            d
        };
        match decision {
            Decision::Serve => {
                // An error here means we raced a rename; poll again.
                if let Ok(bytes) = tokio::fs::read(&path).await {
                    return Ok(bytes);
                }
            }
            Decision::GiveUp => return Err(SegmentError::Superseded),
            Decision::Wait | Decision::Restart => {}
        }
        if Instant::now() >= deadline {
            return Err(SegmentError::Timeout);
        }
        tokio::time::sleep(POLL).await;
    }
}

/// Background tick: keeps throttling and failure handling going while no request waits.
pub fn spawn_supervisor(session: Weak<Session>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(SUPERVISOR_TICK).await;
            let Some(s) = session.upgrade() else { return };
            if s.is_closed() {
                return;
            }
            lock(&s.hls).poll(&s);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use PreviewEncoder::*;

    const JOB: Option<JobView> = Some(JobView { start: 10, next: 13 });

    #[test]
    fn existing_segment_is_served() {
        assert_eq!(decide(3, true, None, false), Decision::Serve);
        assert_eq!(decide(50, true, JOB, true), Decision::Serve);
    }

    #[test]
    fn segment_in_job_window_waits() {
        assert_eq!(decide(10, false, JOB, true), Decision::Wait);
        assert_eq!(decide(13, false, JOB, true), Decision::Wait);
        assert_eq!(decide(17, false, JOB, true), Decision::Wait); // next + 4
        assert_eq!(decide(17, false, JOB, false), Decision::Wait);
    }

    #[test]
    fn segment_outside_window_restarts_only_for_latest() {
        assert_eq!(decide(18, false, JOB, true), Decision::Restart);
        assert_eq!(decide(9, false, JOB, true), Decision::Restart);
        assert_eq!(decide(9, false, None, true), Decision::Restart);
        assert_eq!(decide(18, false, JOB, false), Decision::GiveUp);
        assert_eq!(decide(0, false, None, false), Decision::GiveUp);
    }

    #[test]
    fn throttle_beyond_twelve_ahead() {
        assert!(!should_throttle(22, 10));
        assert!(should_throttle(23, 10));
        assert!(!should_throttle(5, 10));
    }

    #[test]
    fn fallback_chain() {
        assert_eq!(fallback(true, Nvenc), Some((false, Nvenc)));
        assert_eq!(fallback(false, Nvenc), Some((false, X264)));
        assert_eq!(fallback(true, X264), Some((false, X264)));
        assert_eq!(fallback(false, X264), None);
    }
}
