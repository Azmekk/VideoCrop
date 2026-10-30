//! Finds a user-installed ffmpeg on PATH and checks it can do everything VideoCrop asks of it
//! (encoders + a recent enough version). The verdict is cached per binary (size + mtime) in
//! `tools-check.json`, so only the first launch with a new ffmpeg pays the ~100 ms probe.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use super::tools::{FFMPEG_EXE, FFPROBE_EXE};

/// Encoders the presets and the custom panel can select (aac is built into every ffmpeg).
pub const REQUIRED_ENCODERS: [&str; 6] = ["libx264", "libx265", "libsvtav1", "libvpx-vp9", "libopus", "libmp3lame"];
/// Oldest release with everything the HLS preview relies on.
pub const MIN_MAJOR_VERSION: u32 = 6;

const CACHE_FILE: &str = "tools-check.json";

/// First directory on PATH holding both ffmpeg.exe and ffprobe.exe, with scoop shims resolved
/// to the real binaries (killing a shim would leave its ffmpeg child running).
pub fn find_on_path() -> Option<(PathBuf, PathBuf)> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|dir| {
        let ffmpeg = dir.join(FFMPEG_EXE);
        let ffprobe = dir.join(FFPROBE_EXE);
        (ffmpeg.is_file() && ffprobe.is_file()).then(|| (resolve_shim(&ffmpeg), resolve_shim(&ffprobe)))
    })
}

/// scoop puts `ffmpeg.shim` (`path = "C:\...\ffmpeg.exe"`) next to its `ffmpeg.exe` shim.
fn resolve_shim(exe: &Path) -> PathBuf {
    fs::read_to_string(exe.with_extension("shim"))
        .ok()
        .and_then(|text| shim_target(&text))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| exe.to_path_buf())
}

pub fn shim_target(shim_text: &str) -> Option<PathBuf> {
    shim_text.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == "path").then(|| PathBuf::from(value.trim().trim_matches('"')))
    })
}

/// None = good enough; Some(reason) = offer the download instead.
pub fn check(ffmpeg: &Path, config_dir: &Path) -> Option<String> {
    let stamp = file_stamp(ffmpeg);
    let cache_path = config_dir.join(CACHE_FILE);
    let key = ffmpeg.to_string_lossy().into_owned();
    if let (Some((size, modified)), Some(cached)) = (stamp, read_cache(&cache_path)) {
        if cached.ffmpeg == key && cached.size == size && cached.modified == modified {
            return cached.problem;
        }
    }

    let problem = probe(ffmpeg);
    if let Some((size, modified)) = stamp {
        let entry = CacheEntry { ffmpeg: key, size, modified, problem: problem.clone() };
        if let Ok(json) = serde_json::to_vec_pretty(&entry) {
            let _ = fs::create_dir_all(config_dir);
            let _ = fs::write(&cache_path, json);
        }
    }
    problem
}

fn probe(ffmpeg: &Path) -> Option<String> {
    let shown = ffmpeg.display();
    let Some(version) = run(ffmpeg, &["-hide_banner", "-version"]) else {
        return Some(format!("The ffmpeg on your PATH ({shown}) didn't run."));
    };
    let Some(encoders) = run(ffmpeg, &["-hide_banner", "-encoders"]) else {
        return Some(format!("The ffmpeg on your PATH ({shown}) didn't run."));
    };
    evaluate(&version, &encoders).map(|why| format!("The ffmpeg on your PATH ({shown}) {why}."))
}

/// Pure verdict over `ffmpeg -version` and `ffmpeg -encoders` output.
pub fn evaluate(version_output: &str, encoders_output: &str) -> Option<String> {
    if let Some(major) = parse_major_version(version_output) {
        if major < MIN_MAJOR_VERSION {
            return Some(format!("is version {major}; VideoCrop needs {MIN_MAJOR_VERSION} or newer"));
        }
    }
    let missing = missing_encoders(encoders_output);
    if !missing.is_empty() {
        return Some(format!("is missing {} (VideoCrop needs a full GPL build)", missing.join(", ")));
    }
    None
}

/// `ffmpeg version 7.1.1-full_build…` / `ffmpeg version n6.0` -> major. Git builds
/// (`N-118…-g…`) and anything unrecognised return None and are treated as new enough.
pub fn parse_major_version(version_output: &str) -> Option<u32> {
    let rest = version_output.lines().next()?.trim().strip_prefix("ffmpeg version ")?;
    let digits: String = rest.trim_start_matches('n').chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// Encoder lines look like ` V....D libx264  libx264 H.264 / AVC ...`: the name is the 2nd column.
pub fn missing_encoders(encoders_output: &str) -> Vec<&'static str> {
    let available: Vec<&str> = encoders_output.lines().filter_map(|l| l.split_whitespace().nth(1)).collect();
    REQUIRED_ENCODERS.into_iter().filter(|e| !available.contains(e)).collect()
}

fn run(exe: &Path, args: &[&str]) -> Option<String> {
    let mut cmd = Command::new(exe);
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let out = cmd.output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn file_stamp(path: &Path) -> Option<(u64, u64)> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_secs();
    Some((meta.len(), modified))
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    ffmpeg: String,
    size: u64,
    modified: u64,
    problem: Option<String>,
}

fn read_cache(path: &Path) -> Option<CacheEntry> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENCODERS_FULL: &str = "Encoders:\n V..... = Video\n ------\n V....D libx264              libx264 H.264\n V....D libx265              libx265 H.265\n V....D libsvtav1            SVT-AV1\n V....D libvpx-vp9           libvpx VP9\n A....D aac                  AAC\n A....D libopus              libopus Opus\n A....D libmp3lame           libmp3lame MP3\n";

    #[test]
    fn versions() {
        assert_eq!(parse_major_version("ffmpeg version 7.1.1-full_build-www.gyan.dev Copyright"), Some(7));
        assert_eq!(parse_major_version("ffmpeg version n6.0 Copyright"), Some(6));
        assert_eq!(parse_major_version("ffmpeg version 4.4.2-0ubuntu0.22.04.1"), Some(4));
        assert_eq!(parse_major_version("ffmpeg version N-118436-g5c6a3e8 Copyright"), None);
        assert_eq!(parse_major_version("garbage"), None);
    }

    #[test]
    fn full_build_passes() {
        assert_eq!(evaluate("ffmpeg version 7.1", ENCODERS_FULL), None);
        assert_eq!(evaluate("ffmpeg version N-1-gabc", ENCODERS_FULL), None);
    }

    #[test]
    fn old_version_fails() {
        let why = evaluate("ffmpeg version 5.1.2", ENCODERS_FULL).unwrap();
        assert!(why.contains("version 5"), "{why}");
    }

    #[test]
    fn missing_encoders_are_listed() {
        let lgpl = ENCODERS_FULL.replace("libx264 ", "h264_mf ").replace("libx265 ", "hevc_mf ");
        assert_eq!(missing_encoders(&lgpl), vec!["libx264", "libx265"]);
        let why = evaluate("ffmpeg version 7.0", &lgpl).unwrap();
        assert!(why.contains("libx264, libx265"), "{why}");
    }

    /// Checks the real ffmpeg on this machine's PATH: `cargo test path_ffmpeg -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn path_ffmpeg_on_this_machine() {
        let (ffmpeg, ffprobe) = find_on_path().expect("no ffmpeg + ffprobe on PATH");
        println!("ffmpeg: {}
ffprobe: {}", ffmpeg.display(), ffprobe.display());
        let dir = std::env::temp_dir().join("videocrop-toolcheck-test");
        let _ = fs::remove_dir_all(&dir);
        let started = std::time::Instant::now();
        println!("verdict: {:?} in {:?}", check(&ffmpeg, &dir), started.elapsed());
        let started = std::time::Instant::now();
        println!("cached verdict: {:?} in {:?}", check(&ffmpeg, &dir), started.elapsed());
    }

    #[test]
    fn shim_files() {
        assert_eq!(
            shim_target("path = \"C:\\Users\\me\\scoop\\apps\\ffmpeg\\current\\bin\\ffmpeg.exe\"\n"),
            Some(PathBuf::from(r"C:\Users\me\scoop\apps\ffmpeg\current\bin\ffmpeg.exe"))
        );
        assert_eq!(shim_target("path = C:\\x\\ffmpeg.exe\nargs = -y"), Some(PathBuf::from(r"C:\x\ffmpeg.exe")));
        assert_eq!(shim_target("args = -y"), None);
    }
}
