//! Turns a `JobDraft` into a fully resolved `JobPlan`: output path, effective spec (platform
//! presets resolved for this source/cut), effective resize, final dimensions, size estimate,
//! warnings and the human-readable pipeline summary.

use std::path::Path;
use super::models::*;

pub fn plan(info: &VideoInfo, draft: &JobDraft, catalog: &PresetCatalog, exists: impl Fn(&Path) -> bool) -> Result<JobPlan, String> {
    let _ = (info, draft, catalog, exists);
    todo!("core agent")
}
