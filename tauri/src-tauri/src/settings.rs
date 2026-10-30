//! `settings.json` in the app config dir. Load-or-default; atomic write (temp file + rename).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::core::models::Settings;

pub const MAX_RECENT: usize = 10;

pub fn file_path(config_dir: &Path) -> PathBuf {
    config_dir.join("settings.json")
}

/// Missing or corrupt file -> defaults (a corrupt file is overwritten on the next save).
pub fn load(path: &Path) -> Settings {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Settings>(&bytes).ok())
        .unwrap_or_default()
}

pub fn save(path: &Path, settings: &Settings) -> Result<(), String> {
    let json = serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("Could not create settings folder: {e}"))?;
    }
    let tmp = path.with_extension("json.tmp");
    let write = || -> std::io::Result<()> {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(&json)?;
        f.sync_all()?;
        drop(f);
        // On Windows std's rename uses MoveFileExW(REPLACE_EXISTING): atomic replace.
        fs::rename(&tmp, path)
    };
    write().map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("Could not save settings: {e}")
    })
}

/// Moves `path` to the front of the recent list (case-insensitive dedup), capped at 10.
pub fn push_recent(settings: &mut Settings, path: &str) {
    settings.recent_files.retain(|p| !p.eq_ignore_ascii_case(path));
    settings.recent_files.insert(0, path.to_owned());
    settings.recent_files.truncate(MAX_RECENT);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_dedups_and_caps() {
        let mut s = Settings::default();
        for i in 0..12 {
            push_recent(&mut s, &format!("C:\\v{i}.mp4"));
        }
        push_recent(&mut s, "c:\\V5.MP4");
        assert_eq!(s.recent_files.len(), MAX_RECENT);
        assert_eq!(s.recent_files[0], "c:\\V5.MP4");
        assert_eq!(s.recent_files.iter().filter(|p| p.eq_ignore_ascii_case("C:\\v5.mp4")).count(), 1);
    }

    #[test]
    fn save_then_load_roundtrip_and_corrupt_falls_back() {
        let dir = std::env::temp_dir().join(format!("vc-settings-test-{}", std::process::id()));
        let path = file_path(&dir);
        let s = Settings { theme: "dark".into(), preview_encoder: Some("libx264".into()), ..Default::default() };
        save(&path, &s).unwrap();
        assert_eq!(load(&path), s);
        fs::write(&path, b"{not json").unwrap();
        assert_eq!(load(&path), Settings::default());
        let _ = fs::remove_dir_all(&dir);
    }
}
