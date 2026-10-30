//! Locate and download ffmpeg/ffprobe. Port of `ToolDownloadService.cs` (mpv no longer needed).
//!
//! Resolution order (each tool independently): `VIDEOCROP_TOOLS_DIR` (dev override),
//! `<app_local_data_dir>/tools` (where downloads land), `tools/` next to the exe.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::core::models::{ToolDownloadProgress, ToolStatus};
use crate::state::{AppState, Paths};

pub const ENV_TOOLS_DIR: &str = "VIDEOCROP_TOOLS_DIR";
pub const FFMPEG_EXE: &str = "ffmpeg.exe";
pub const FFPROBE_EXE: &str = "ffprobe.exe";
const LICENSE_NAME: &str = "ffmpeg-LICENSE.txt";
const USER_AGENT: &str = concat!("VideoCrop/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolPaths {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
}

pub fn candidate_dirs(paths: &Paths) -> Vec<PathBuf> {
    let mut dirs = Vec::with_capacity(3);
    if let Some(env) = std::env::var_os(ENV_TOOLS_DIR).filter(|v| !v.is_empty()) {
        dirs.push(PathBuf::from(env));
    }
    dirs.push(paths.tools_dir.clone());
    if let Some(d) = &paths.exe_tools_dir {
        dirs.push(d.clone());
    }
    dirs
}

fn find_in(dirs: &[PathBuf], exe: &str) -> Option<PathBuf> {
    dirs.iter().map(|d| d.join(exe)).find(|p| p.is_file())
}

/// Resolves both tools (uses the cache when present).
pub fn status(state: &AppState) -> ToolStatus {
    let tools_dir = state.paths.tools_dir.to_string_lossy().into_owned();
    if let Some(t) = state.tools.read().unwrap_or_else(|e| e.into_inner()).clone() {
        return ToolStatus {
            ffmpeg: Some(t.ffmpeg.to_string_lossy().into_owned()),
            ffprobe: Some(t.ffprobe.to_string_lossy().into_owned()),
            ready: true,
            tools_dir,
        };
    }
    let dirs = candidate_dirs(&state.paths);
    let ffmpeg = find_in(&dirs, FFMPEG_EXE);
    let ffprobe = find_in(&dirs, FFPROBE_EXE);
    if let (Some(ffmpeg), Some(ffprobe)) = (&ffmpeg, &ffprobe) {
        *state.tools.write().unwrap_or_else(|e| e.into_inner()) =
            Some(ToolPaths { ffmpeg: ffmpeg.clone(), ffprobe: ffprobe.clone() });
    }
    ToolStatus {
        ready: ffmpeg.is_some() && ffprobe.is_some(),
        ffmpeg: ffmpeg.map(|p| p.to_string_lossy().into_owned()),
        ffprobe: ffprobe.map(|p| p.to_string_lossy().into_owned()),
        tools_dir,
    }
}

pub fn require(state: &AppState) -> Result<ToolPaths, String> {
    status(state);
    state
        .tools
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .ok_or_else(|| "ffmpeg/ffprobe are not installed yet.".to_string())
}

/// Downloads the BtbN GPL build and extracts ffmpeg/ffprobe/LICENSE into `tools_dir`.
/// Emits `tools://progress`. Blocking work runs on the blocking pool.
pub async fn download(app: AppHandle, state: AppState) -> Result<ToolStatus, String> {
    if state.downloading.swap(true, Ordering::SeqCst) {
        return Err("A download is already in progress.".into());
    }
    struct Reset<'a>(&'a AppState);
    impl Drop for Reset<'_> {
        fn drop(&mut self) {
            self.0.downloading.store(false, Ordering::SeqCst);
        }
    }
    let _reset = Reset(&state);

    let dir = state.paths.tools_dir.clone();
    let emit_app = app.clone();
    let report = move |fraction: f64, status: String| {
        let _ = emit_app.emit("tools://progress", ToolDownloadProgress { fraction, status });
    };
    tauri::async_runtime::spawn_blocking(move || download_blocking(&dir, &report))
        .await
        .map_err(|e| format!("Download task failed: {e}"))??;

    *state.tools.write().unwrap_or_else(|e| e.into_inner()) = None;
    Ok(status(&state))
}

pub fn archive_url() -> String {
    let arch = if cfg!(target_arch = "aarch64") { "winarm64" } else { "win64" };
    format!("https://github.com/BtbN/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-{arch}-gpl.zip")
}

fn download_blocking(tools_dir: &Path, report: &dyn Fn(f64, String)) -> Result<(), String> {
    fs::create_dir_all(tools_dir).map_err(|e| format!("Could not create {}: {e}", tools_dir.display()))?;
    let temp_zip = std::env::temp_dir().join(format!("videocrop-ffmpeg-{:016x}.zip", rand::random::<u64>()));
    let result = (|| {
        fetch(&archive_url(), &temp_zip, report)?;
        report(0.9, "Extracting…".into());
        extract(&temp_zip, tools_dir)?;
        report(1.0, "Done".into());
        Ok(())
    })();
    let _ = fs::remove_file(&temp_zip);
    result
}

fn fetch(url: &str, dest: &Path, report: &dyn Fn(f64, String)) -> Result<(), String> {
    report(0.0, "Connecting…".into());
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_recv_response(Some(Duration::from_secs(30)))
        .timeout_per_read(Some(Duration::from_secs(60)))
        .user_agent(USER_AGENT)
        .build()
        .into();
    let mut resp = agent.get(url).call().map_err(|e| format!("Download failed: {e}"))?;
    let total = resp.body().content_length().unwrap_or(0);
    let mut src = resp.body_mut().as_reader();
    let mut out = io::BufWriter::new(fs::File::create(dest).map_err(|e| format!("Could not write temp file: {e}"))?);

    let mut buf = vec![0u8; 64 * 1024];
    let mut copied: u64 = 0;
    let mut last = 0.0;
    loop {
        let n = src.read(&mut buf).map_err(|e| format!("Download interrupted: {e}"))?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n]).map_err(|e| format!("Could not write temp file: {e}"))?;
        copied += n as u64;
        if total > 0 {
            // 0..0.9 download, 0.9..1.0 extract.
            let fraction = (copied as f64 / total as f64 * 0.9).clamp(0.0, 0.9);
            if fraction - last >= 0.01 {
                report(fraction, format_status(copied, total));
                last = fraction;
            }
        }
    }
    out.flush().map_err(|e| format!("Could not write temp file: {e}"))?;
    if total > 0 && copied != total {
        return Err(format!("Download incomplete ({copied} of {total} bytes)."));
    }
    Ok(())
}

pub fn format_status(copied: u64, total: u64) -> String {
    let mb = |b: u64| b as f64 / 1024.0 / 1024.0;
    format!("{:.1} / {:.1} MB", mb(copied), mb(total))
}

/// Which archive entry maps to which installed file name (None = skip).
pub fn install_name(entry_path: &str) -> Option<&'static str> {
    let name = entry_path.rsplit(['/', '\\']).next().unwrap_or(entry_path);
    if name.eq_ignore_ascii_case(FFMPEG_EXE) {
        Some(FFMPEG_EXE)
    } else if name.eq_ignore_ascii_case(FFPROBE_EXE) {
        Some(FFPROBE_EXE)
    } else if name.len() >= 7 && name[..7].eq_ignore_ascii_case("LICENSE") {
        Some(LICENSE_NAME)
    } else {
        None
    }
}

fn extract(zip_path: &Path, tools_dir: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(io::BufReader::new(file)).map_err(|e| format!("Bad archive: {e}"))?;
    let mut got_ffmpeg = false;
    let mut got_ffprobe = false;
    let mut got_license = false;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| format!("Bad archive: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        let Some(target) = install_name(entry.name()) else { continue };
        let flag = match target {
            FFMPEG_EXE => &mut got_ffmpeg,
            FFPROBE_EXE => &mut got_ffprobe,
            _ => &mut got_license,
        };
        if *flag {
            continue;
        }
        let final_path = tools_dir.join(target);
        let part = tools_dir.join(format!("{target}.part"));
        let mut write = || -> io::Result<()> {
            let mut out = fs::File::create(&part)?;
            io::copy(&mut entry, &mut out)?;
            out.sync_all()?;
            drop(out);
            fs::rename(&part, &final_path)
        };
        if let Err(e) = write() {
            let _ = fs::remove_file(&part);
            // The license is best effort, the executables are not.
            if target != LICENSE_NAME {
                return Err(format!("Could not install {target}: {e}"));
            }
            continue;
        }
        *flag = true;
    }
    match (got_ffmpeg, got_ffprobe) {
        (true, true) => Ok(()),
        (false, _) => Err("ffmpeg.exe not found in archive".into()),
        (_, false) => Err("ffprobe.exe not found in archive".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_names() {
        assert_eq!(install_name("ffmpeg-master-latest-win64-gpl/bin/ffmpeg.exe"), Some(FFMPEG_EXE));
        assert_eq!(install_name("x/bin/FFPROBE.EXE"), Some(FFPROBE_EXE));
        assert_eq!(install_name("x/LICENSE.txt"), Some(LICENSE_NAME));
        assert_eq!(install_name("x/bin/ffplay.exe"), None);
        assert_eq!(install_name("x/doc/ffmpeg.html"), None);
    }

    #[test]
    fn url_uses_btbn_gpl() {
        let url = archive_url();
        assert!(url.starts_with("https://github.com/BtbN/FFmpeg-Builds/releases/latest/download/"));
        assert!(url.ends_with("-gpl.zip"));
    }

    #[test]
    fn status_text() {
        assert_eq!(format_status(1024 * 1024, 10 * 1024 * 1024), "1.0 / 10.0 MB");
    }
}
