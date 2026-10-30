//! Output filenames: `<base>_VideoCrop.<ext>`, then `_VideoCrop2`, `_VideoCrop3`, ...
//! An existing `_VideoCrop[N]` suffix on the source is stripped first.

use std::path::{Path, PathBuf};

/// `exists` is injected so this stays pure and testable.
pub fn next_available(source_path: &str, output_dir: &str, extension: &str, exists: impl Fn(&Path) -> bool) -> Result<PathBuf, String> {
    let _ = (source_path, output_dir, extension, exists);
    todo!("core agent")
}
