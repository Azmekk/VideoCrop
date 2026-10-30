//! Output filenames: `<base>_VideoCrop.<ext>`, then `_VideoCrop2`, `_VideoCrop3`, ...
//! An existing `_VideoCrop[N]` suffix on the source is stripped first.

use std::path::{Path, PathBuf};

pub const SUFFIX: &str = "_VideoCrop";

/// `exists` is injected so this stays pure and testable.
/// `extension` may be given with or without the leading dot.
pub fn next_available(source_path: &str, output_dir: &str, extension: &str, exists: impl Fn(&Path) -> bool) -> Result<PathBuf, String> {
    if source_path.trim().is_empty() {
        return Err("Source path required.".into());
    }
    if output_dir.trim().is_empty() {
        return Err("Output directory required.".into());
    }
    let ext = extension.trim().trim_start_matches('.');
    if ext.is_empty() {
        return Err("Extension required.".into());
    }

    let raw_base = Path::new(source_path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "output".to_string());
    let base = strip_existing_suffix(&raw_base);
    let dir = Path::new(output_dir);

    let first = dir.join(format!("{base}{SUFFIX}.{ext}"));
    if !exists(&first) {
        return Ok(first);
    }
    for i in 2..u32::MAX {
        let candidate = dir.join(format!("{base}{SUFFIX}{i}.{ext}"));
        if !exists(&candidate) {
            return Ok(candidate);
        }
    }
    Err("Could not allocate a unique output filename.".into())
}

/// `clip_VideoCrop` / `clip_VideoCrop3` -> `clip`; anything else is returned unchanged.
pub fn strip_existing_suffix(name: &str) -> &str {
    match name.rfind(SUFFIX) {
        None => name,
        Some(idx) => {
            let tail = &name[idx + SUFFIX.len()..];
            if tail.chars().all(|c| c.is_ascii_digit()) {
                &name[..idx]
            } else {
                name
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn name(p: &Path) -> String {
        p.file_name().unwrap().to_string_lossy().into_owned()
    }

    fn with_existing(existing: &[&str]) -> impl Fn(&Path) -> bool {
        let set: HashSet<String> = existing.iter().map(|s| s.to_string()).collect();
        move |p: &Path| set.contains(&name(p))
    }

    const DIR: &str = "C:/out";

    #[test]
    fn first_call_uses_videocrop_suffix() {
        let r = next_available("C:/in/movie.mp4", DIR, ".mp4", |_| false).unwrap();
        assert_eq!(name(&r), "movie_VideoCrop.mp4");
        assert_eq!(r, Path::new(DIR).join("movie_VideoCrop.mp4"));
    }

    #[test]
    fn collision_with_base_uses_index_2() {
        let r = next_available("C:/in/movie.mp4", DIR, ".mp4", with_existing(&["movie_VideoCrop.mp4"])).unwrap();
        assert_eq!(name(&r), "movie_VideoCrop2.mp4");
    }

    #[test]
    fn collision_with_base_and_index_2_uses_index_3() {
        let r = next_available("C:/in/movie.mp4", DIR, ".mp4", with_existing(&["movie_VideoCrop.mp4", "movie_VideoCrop2.mp4"])).unwrap();
        assert_eq!(name(&r), "movie_VideoCrop3.mp4");
    }

    #[test]
    fn gap_in_indices_takes_first_free() {
        let r = next_available("C:/in/movie.mp4", DIR, "mp4", with_existing(&["movie_VideoCrop.mp4", "movie_VideoCrop3.mp4"])).unwrap();
        assert_eq!(name(&r), "movie_VideoCrop2.mp4");
    }

    #[test]
    fn collision_only_counts_same_extension() {
        let r = next_available("C:/in/movie.mp4", DIR, "webm", with_existing(&["movie_VideoCrop.mp4"])).unwrap();
        assert_eq!(name(&r), "movie_VideoCrop.webm");
    }

    #[test]
    fn source_already_ending_in_videocrop_does_not_double_suffix() {
        let r = next_available("C:/in/clip_VideoCrop.mp4", DIR, ".mp4", |_| false).unwrap();
        assert_eq!(name(&r), "clip_VideoCrop.mp4");
    }

    #[test]
    fn source_already_ending_in_videocrop3_does_not_double_suffix() {
        let r = next_available("C:/in/clip_VideoCrop3.mp4", DIR, ".mp4", |_| false).unwrap();
        assert_eq!(name(&r), "clip_VideoCrop.mp4");
    }

    #[test]
    fn suffixed_source_still_collides() {
        let r = next_available("C:/in/clip_VideoCrop2.mp4", DIR, ".mp4", with_existing(&["clip_VideoCrop.mp4", "clip_VideoCrop2.mp4"])).unwrap();
        assert_eq!(name(&r), "clip_VideoCrop3.mp4");
    }

    #[test]
    fn suffix_in_middle_is_kept() {
        let r = next_available("C:/in/clip_VideoCrop_final.mp4", DIR, "mp4", |_| false).unwrap();
        assert_eq!(name(&r), "clip_VideoCrop_final_VideoCrop.mp4");
    }

    #[test]
    fn lowercase_suffix_is_not_stripped() {
        let r = next_available("C:/in/clip_videocrop.mp4", DIR, "mp4", |_| false).unwrap();
        assert_eq!(name(&r), "clip_videocrop_VideoCrop.mp4");
    }

    #[test]
    fn only_last_suffix_stripped() {
        assert_eq!(strip_existing_suffix("a_VideoCrop_VideoCrop2"), "a_VideoCrop");
        assert_eq!(strip_existing_suffix("_VideoCrop"), "");
        assert_eq!(strip_existing_suffix("plain"), "plain");
    }

    #[test]
    fn unicode_filename_handled() {
        let r = next_available("C:/in/日本語.mp4", DIR, ".mp4", |_| false).unwrap();
        assert_eq!(name(&r), "日本語_VideoCrop.mp4");
        let r = next_available("C:/in/clip 🎬_VideoCrop4.mkv", DIR, "mkv", |_| false).unwrap();
        assert_eq!(name(&r), "clip 🎬_VideoCrop.mkv");
    }

    #[test]
    fn extension_with_no_dot_works() {
        let r = next_available("C:/in/clip.mp4", DIR, "webm", |_| false).unwrap();
        assert_eq!(name(&r), "clip_VideoCrop.webm");
    }

    #[test]
    fn multi_dot_source_keeps_inner_dots() {
        let r = next_available("C:/in/my.clip.v2.mov", DIR, "mp4", |_| false).unwrap();
        assert_eq!(name(&r), "my.clip.v2_VideoCrop.mp4");
    }

    #[test]
    fn source_without_extension() {
        let r = next_available("C:/in/rawfile", DIR, "mp4", |_| false).unwrap();
        assert_eq!(name(&r), "rawfile_VideoCrop.mp4");
    }

    #[test]
    fn empty_arguments_error() {
        assert!(next_available("", DIR, "mp4", |_| false).is_err());
        assert!(next_available("C:/in/a.mp4", " ", "mp4", |_| false).is_err());
        assert!(next_available("C:/in/a.mp4", DIR, ".", |_| false).is_err());
        assert!(next_available("C:/in/a.mp4", DIR, "", |_| false).is_err());
    }
}
