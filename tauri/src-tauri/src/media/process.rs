//! Child-process helper. Every child:
//! - gets its arguments as a list (never a shell string),
//! - is created with `CREATE_NO_WINDOW` so no console flashes,
//! - is assigned to one process-wide Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, so
//!   ffmpeg dies with the app even on a crash,
//! - is `kill_on_drop`, so dropping the `Child` (e.g. a cancelled future) kills it.

use std::collections::VecDeque;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader};
use tokio::process::{Child, Command};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pipe {
    Null,
    Piped,
}

/// Spawns `program args...` with the conventions above. stdin is always null.
pub fn spawn<S: AsRef<OsStr>>(program: &Path, args: &[S], stdout: Pipe, stderr: Pipe) -> io::Result<Child> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(stdio(stdout))
        .stderr(stdio(stderr))
        .kill_on_drop(true);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let child = cmd.spawn()?;
    #[cfg(windows)]
    job::assign(&child);
    Ok(child)
}

fn stdio(p: Pipe) -> Stdio {
    match p {
        Pipe::Null => Stdio::null(),
        Pipe::Piped => Stdio::piped(),
    }
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug)]
pub struct Captured {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    /// Last `STDERR_TAIL` bytes of stderr.
    pub stderr: String,
}

pub const STDERR_TAIL: usize = 4096;

/// Runs to completion collecting stdout fully and the tail of stderr. Dropping the returned
/// future kills the child. `timeout` None = no limit.
pub async fn run_capture<S: AsRef<OsStr>>(program: &Path, args: &[S], timeout: Option<Duration>) -> io::Result<Captured> {
    let mut child = spawn(program, args, Pipe::Piped, Pipe::Piped)?;
    let mut out = child.stdout.take().expect("piped stdout");
    let err = child.stderr.take().expect("piped stderr");
    let work = async {
        let mut stdout = Vec::new();
        let (r_out, stderr) = tokio::join!(out.read_to_end(&mut stdout), tail_lines(err, STDERR_TAIL));
        r_out?;
        let status = child.wait().await?;
        Ok::<_, io::Error>(Captured { status, stdout, stderr })
    };
    match timeout {
        Some(t) => tokio::time::timeout(t, work)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "process timed out"))?,
        None => work.await,
    }
}

/// Reads `reader` line by line, returning the last `max_bytes` (approximately, whole lines).
pub async fn tail_lines<R: AsyncRead + Unpin>(reader: R, max_bytes: usize) -> String {
    let mut tail = TailBuffer::new(max_bytes);
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        tail.push(&line);
    }
    tail.into_string()
}

/// Keeps the newest lines whose combined size is <= `max` bytes.
#[derive(Debug)]
pub struct TailBuffer {
    lines: VecDeque<String>,
    bytes: usize,
    max: usize,
}

impl TailBuffer {
    pub fn new(max: usize) -> Self {
        Self { lines: VecDeque::new(), bytes: 0, max }
    }

    pub fn push(&mut self, line: &str) {
        let line = if line.len() + 1 > self.max {
            // Keep the end of an absurdly long line, on a char boundary.
            let mut start = (line.len() + 1).saturating_sub(self.max).min(line.len());
            while !line.is_char_boundary(start) {
                start += 1;
            }
            &line[start..]
        } else {
            line
        };
        self.bytes += line.len() + 1;
        self.lines.push_back(line.to_owned());
        while self.bytes > self.max {
            match self.lines.pop_front() {
                Some(l) => self.bytes -= l.len() + 1,
                None => break,
            }
        }
    }

    pub fn as_string(&self) -> String {
        self.lines.iter().map(String::as_str).collect::<Vec<_>>().join("\n")
    }

    pub fn into_string(self) -> String {
        self.as_string()
    }
}

#[cfg(windows)]
mod job {
    use std::sync::OnceLock;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    /// The job handle as an integer (raw pointers aren't Send/Sync). 0 = creation failed.
    /// Intentionally never closed: the OS closes it at process exit, which kills the children.
    static JOB: OnceLock<usize> = OnceLock::new();

    fn handle() -> HANDLE {
        *JOB.get_or_init(|| unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return 0;
            }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const core::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            job as usize
        }) as HANDLE
    }

    pub fn assign(child: &tokio::process::Child) {
        let job = handle();
        if job.is_null() {
            return;
        }
        if let Some(h) = child.raw_handle() {
            // Best effort: failure only means the child might outlive a crash.
            unsafe { AssignProcessToJobObject(job, h as HANDLE) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_buffer_keeps_newest_lines() {
        let mut t = TailBuffer::new(10);
        t.push("aaaa");
        t.push("bbbb");
        t.push("cccc");
        assert_eq!(t.as_string(), "bbbb\ncccc");
    }

    #[test]
    fn tail_buffer_truncates_huge_line() {
        let mut t = TailBuffer::new(4);
        t.push("123456789");
        assert!(t.as_string().len() <= 4);
    }

    #[tokio::test]
    async fn run_capture_collects_stdout() {
        let cmd = Path::new("cmd.exe");
        let out = run_capture(cmd, &["/C", "echo hello"], Some(Duration::from_secs(10))).await.unwrap();
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hello");
    }
}
