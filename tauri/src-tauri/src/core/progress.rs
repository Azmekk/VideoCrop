//! Stateful parser for ffmpeg `-progress pipe:1` key=value lines. One instance per process.

use super::models::EncodeProgress;

#[derive(Debug, Default)]
pub struct ProgressParser {
    frame: u64,
    fps: f64,
    out_time_sec: f64,
    speed: f64,
}

impl ProgressParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a snapshot on every `progress=` line (finished when `progress=end`).
    pub fn on_line(&mut self, line: &str) -> Option<EncodeProgress> {
        let _ = (line, self.frame, self.fps, self.out_time_sec, self.speed);
        todo!("core agent")
    }
}
