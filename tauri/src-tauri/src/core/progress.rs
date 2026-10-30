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
    /// Unparseable values (e.g. `N/A`) keep the previous value.
    pub fn on_line(&mut self, line: &str) -> Option<EncodeProgress> {
        let line = line.trim_end_matches(['\r', '\n']);
        let (key, value) = line.split_once('=')?;
        if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphabetic() || c == '_') {
            return None;
        }
        let value = value.trim();
        match key {
            "frame" => {
                if let Ok(f) = value.parse::<u64>() {
                    self.frame = f;
                }
            }
            "fps" => {
                if let Some(f) = parse_f64(value) {
                    self.fps = f;
                }
            }
            // ffmpeg's out_time_ms is (despite the name) also microseconds.
            "out_time_us" | "out_time_ms" => {
                if let Ok(us) = value.parse::<i64>() {
                    self.out_time_sec = (us.max(0)) as f64 / 1_000_000.0;
                }
            }
            "out_time" => {
                if let Some(t) = parse_hms(value) {
                    self.out_time_sec = t.max(0.0);
                }
            }
            "speed" => {
                let stripped = value.strip_suffix(['x', 'X']).unwrap_or(value).trim();
                if let Some(s) = parse_f64(stripped) {
                    self.speed = s;
                }
            }
            "progress" => {
                return Some(EncodeProgress {
                    frame: self.frame,
                    fps: self.fps,
                    out_time_sec: self.out_time_sec,
                    speed: self.speed,
                    finished: value == "end",
                });
            }
            _ => {}
        }
        None
    }
}

fn parse_f64(s: &str) -> Option<f64> {
    s.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// `HH:MM:SS.micro` (hours may exceed 24, optional leading `-`).
fn parse_hms(s: &str) -> Option<f64> {
    let (neg, body) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    let mut parts = body.split(':');
    let h: f64 = parts.next()?.parse::<u64>().ok()? as f64;
    let m: f64 = parts.next()?.parse::<u64>().ok()? as f64;
    let sec = parse_f64(parts.next()?)?;
    if parts.next().is_some() || m >= 60.0 || !(0.0..60.0).contains(&sec) {
        return None;
    }
    let t = h * 3600.0 + m * 60.0 + sec;
    Some(if neg { -t } else { t })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_progress_continue_block() {
        let mut p = ProgressParser::new();
        assert!(p.on_line("frame=123").is_none());
        assert!(p.on_line("fps=58.2").is_none());
        assert!(p.on_line("out_time_us=4920000").is_none());
        assert!(p.on_line("speed=1.95x").is_none());
        let r = p.on_line("progress=continue").unwrap();
        assert_eq!(r.frame, 123);
        assert!((r.fps - 58.2).abs() < 0.01);
        assert!((r.out_time_sec - 4.92).abs() < 0.001);
        assert!((r.speed - 1.95).abs() < 0.001);
        assert!(!r.finished);
    }

    #[test]
    fn progress_end_marks_finished() {
        let mut p = ProgressParser::new();
        p.on_line("frame=900");
        let r = p.on_line("progress=end").unwrap();
        assert!(r.finished);
        assert_eq!(r.frame, 900);
    }

    #[test]
    fn ignores_unknown_keys() {
        let mut p = ProgressParser::new();
        assert!(p.on_line("bitrate=2000kbps").is_none());
        assert!(p.on_line("garbage line!!").is_none());
        assert!(p.on_line("").is_none());
        assert!(p.on_line("=5").is_none());
        assert!(p.on_line("stream_0_0_q=28.0").is_none());
    }

    #[test]
    fn out_time_ms_is_microseconds() {
        let mut p = ProgressParser::new();
        p.on_line("out_time_ms=2500000");
        assert!((p.on_line("progress=continue").unwrap().out_time_sec - 2.5).abs() < 1e-9);
    }

    #[test]
    fn out_time_string_parsed() {
        let mut p = ProgressParser::new();
        p.on_line("out_time=01:02:03.500000");
        assert!((p.on_line("progress=continue").unwrap().out_time_sec - 3723.5).abs() < 1e-6);
    }

    #[test]
    fn out_time_negative_clamped_and_na_ignored() {
        let mut p = ProgressParser::new();
        p.on_line("out_time_us=1000000");
        p.on_line("out_time=N/A");
        p.on_line("out_time_us=N/A");
        assert!((p.on_line("progress=continue").unwrap().out_time_sec - 1.0).abs() < 1e-9);
        p.on_line("out_time_us=-5000");
        assert_eq!(p.on_line("progress=continue").unwrap().out_time_sec, 0.0);
        p.on_line("out_time=-00:00:01.000000");
        assert_eq!(p.on_line("progress=continue").unwrap().out_time_sec, 0.0);
    }

    #[test]
    fn speed_na_keeps_previous() {
        let mut p = ProgressParser::new();
        p.on_line("speed=2.5x");
        p.on_line("speed=N/A");
        assert!((p.on_line("progress=continue").unwrap().speed - 2.5).abs() < 1e-9);
        p.on_line("speed= 0.75x");
        assert!((p.on_line("progress=continue").unwrap().speed - 0.75).abs() < 1e-9);
    }

    #[test]
    fn crlf_lines_and_state_carries_over() {
        let mut p = ProgressParser::new();
        p.on_line("frame=10\r\n");
        assert_eq!(p.on_line("progress=continue\r").unwrap().frame, 10);
        p.on_line("fps=30");
        let r = p.on_line("progress=continue").unwrap();
        assert_eq!(r.frame, 10);
        assert_eq!(r.fps, 30.0);
    }

    #[test]
    fn full_ffmpeg_block() {
        let block = "frame=240\nfps=59.94\nstream_0_0_q=-1.0\nbitrate=1234.5kbits/s\ntotal_size=1048576\nout_time_us=4004000\nout_time_ms=4004000\nout_time=00:00:04.004000\ndup_frames=0\ndrop_frames=0\nspeed=2.01x\nprogress=continue";
        let mut p = ProgressParser::new();
        let snaps: Vec<_> = block.lines().filter_map(|l| p.on_line(l)).collect();
        assert_eq!(snaps.len(), 1);
        assert_eq!(snaps[0].frame, 240);
        assert!((snaps[0].out_time_sec - 4.004).abs() < 1e-9);
    }
}
