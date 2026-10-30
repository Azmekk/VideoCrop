//! `#[tauri::command]` wrappers. Thin: validation + delegation; errors are user-readable strings.

use std::path::Path;
use std::time::Duration;

use tauri::{AppHandle, State};

use crate::core::models::{JobDraft, JobPlan, OpenedVideo, PresetCatalog, Settings, ToolStatus};
use crate::core::{planner, preview, probe};
use crate::media::session::{PreviewFns, Session};
use crate::media::{encoder, hwdetect, process, protocol, tools};
use crate::settings;
use crate::state::{lock, AppState};
use crate::update;

const PROBE_TIMEOUT: Duration = Duration::from_secs(30);

#[tauri::command]
pub async fn tools_status(state: State<'_, AppState>) -> Result<ToolStatus, String> {
    // May run a PATH ffmpeg once to check its encoders: keep it off the async workers.
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || tools::status(&state))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tools_download(app: AppHandle, state: State<'_, AppState>) -> Result<ToolStatus, String> {
    let state = state.inner().clone();
    let status = tools::download(app, state.clone()).await?;
    if status.ready {
        tauri::async_runtime::spawn(hwdetect::ensure(state));
    }
    Ok(status)
}

#[tauri::command]
pub async fn open_video(path: String, state: State<'_, AppState>) -> Result<OpenedVideo, String> {
    let state = state.inner().clone();
    let t = tools::require(&state)?;
    let meta = std::fs::metadata(&path).map_err(|e| format!("Cannot open {path}: {e}"))?;
    if !meta.is_file() {
        return Err(format!("{path} is not a file."));
    }

    let out = process::run_capture(&t.ffprobe, &probe::ffprobe_args(&path), Some(PROBE_TIMEOUT))
        .await
        .map_err(|e| format!("ffprobe failed: {e}"))?;
    if !out.status.success() {
        let why = out.stderr.lines().last().unwrap_or("unknown error").to_owned();
        return Err(format!("Not a playable media file ({why})."));
    }
    let json = String::from_utf8_lossy(&out.stdout);
    let info = probe::parse_ffprobe_json(&json, &path, Some(meta.len()))?;
    let direct_likely_playable = preview::direct_likely_playable(&info);

    close_current(&state).await;
    let session = Session::create(
        info.clone(),
        t.ffmpeg,
        &state.paths.temp_root,
        state.preview_encoder.clone(),
        PreviewFns::default(),
    )?;
    *lock(&state.session) = Some(session.clone());

    let _ = state.update_settings(|s| settings::push_recent(s, &path));

    let base = format!("{}/{}", protocol::base_url(), session.id);
    Ok(OpenedVideo {
        info,
        session: session.id.clone(),
        direct_url: format!("{base}/direct"),
        hls_url: format!("{base}/hls/index.m3u8"),
        frame_url: format!("{base}/frame"),
        direct_likely_playable,
    })
}

#[tauri::command]
pub async fn close_video(state: State<'_, AppState>) -> Result<(), String> {
    close_current(state.inner()).await;
    Ok(())
}

async fn close_current(state: &AppState) {
    if let Some(s) = state.take_session() {
        s.close().await;
    }
}

#[tauri::command]
pub fn presets(state: State<'_, AppState>) -> PresetCatalog {
    state.catalog().clone()
}

fn plan_for(state: &AppState, draft: &JobDraft) -> Result<(std::sync::Arc<Session>, JobPlan), String> {
    let session = state.current_session().ok_or("Open a video first.")?;
    let plan = planner::plan(&session.info, draft, state.catalog(), |p: &Path| p.exists())?;
    Ok((session, plan))
}

#[tauri::command]
pub async fn plan_job(draft: JobDraft, state: State<'_, AppState>) -> Result<JobPlan, String> {
    plan_for(&state, &draft).map(|(_, plan)| plan)
}

#[tauri::command]
pub async fn encode_start(draft: JobDraft, app: AppHandle, state: State<'_, AppState>) -> Result<JobPlan, String> {
    let state = state.inner().clone();
    let t = tools::require(&state)?;
    // Re-plan here: the planner is the single source of truth, not the frontend's copy.
    let (session, plan) = plan_for(&state, &draft)?;
    encoder::start(app, state.clone(), session, t.ffmpeg, plan.clone())?;
    if let Some(dir) = Path::new(&plan.output_path).parent() {
        let dir = dir.to_string_lossy().into_owned();
        let _ = state.update_settings(|s| s.last_output_dir = Some(dir));
    }
    Ok(plan)
}

#[tauri::command]
pub fn encode_cancel(state: State<'_, AppState>) {
    if let Some(h) = lock(&state.encode).as_ref() {
        h.cancel();
    }
}

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>) -> Settings {
    state.settings_snapshot()
}

#[tauri::command]
pub async fn settings_set(settings: Settings, state: State<'_, AppState>) -> Result<(), String> {
    state.update_settings(|s| {
        // The encoder cache is backend-owned; don't let a stale frontend copy wipe it.
        let preview_encoder = settings.preview_encoder.clone().or_else(|| s.preview_encoder.take());
        *s = settings;
        s.preview_encoder = preview_encoder;
    })
}

#[tauri::command]
pub async fn reveal_in_folder(path: String) -> Result<(), String> {
    if !Path::new(&path).exists() {
        return Err(format!("{path} no longer exists."));
    }
    reveal(&path)
}

#[cfg(windows)]
fn reveal(path: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    // explorer parses its own command line and wants `/select,"<path>"` verbatim. Spawned
    // directly (not via media::process) so the window isn't tied to our kill-on-close job.
    std::process::Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{path}\""))
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Could not open Explorer: {e}"))
}

#[cfg(not(windows))]
fn reveal(path: &str) -> Result<(), String> {
    let dir = Path::new(path).parent().unwrap_or(Path::new("."));
    std::process::Command::new(if cfg!(target_os = "macos") { "open" } else { "xdg-open" })
        .arg(dir)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Latest GitHub release if it is newer than this build; None when up to date or offline.
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<update::UpdateInfo>, String> {
    let current = app.package_info().version.to_string();
    tauri::async_runtime::spawn_blocking(move || update::check(&current))
        .await
        .map_err(|e| e.to_string())?
}

/// Opens the VideoCrop download page in the default browser. No other URL is accepted.
#[tauri::command]
pub async fn open_release_page(url: String) -> Result<(), String> {
    if url != update::DOWNLOAD_PAGE {
        return Err("Refusing to open a URL other than the download page.".into());
    }
    std::process::Command::new("explorer.exe")
        .arg(&url)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Could not open the browser: {e}"))
}
