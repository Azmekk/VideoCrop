pub mod commands;
pub mod core;
pub mod media;
pub mod settings;
pub mod state;
pub mod update;

use std::time::Duration;

use tauri::{Manager, RunEvent};

use crate::media::{hwdetect, protocol, session, tools};
use crate::state::{AppState, Paths};

/// Shows the main window if the frontend hasn't after this long (it normally shows itself
/// after first paint).
const SHOW_FALLBACK: Duration = Duration::from_secs(3);

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .register_asynchronous_uri_scheme_protocol(protocol::SCHEME, protocol::handler)
        .invoke_handler(tauri::generate_handler![
            commands::tools_status,
            commands::tools_download,
            commands::open_video,
            commands::close_video,
            commands::presets,
            commands::plan_job,
            commands::encode_start,
            commands::encode_cancel,
            commands::settings_get,
            commands::settings_set,
            commands::reveal_in_folder,
            commands::check_update,
            commands::open_release_page,
        ])
        .setup(|app| {
            let state = AppState::new(resolve_paths(app.handle())?);
            app.manage(state.clone());

            // Nothing slow on the setup path: sweep / tool check / hwdetect run in background.
            let bg = state.clone();
            tauri::async_runtime::spawn(async move {
                let root = bg.paths.temp_root.clone();
                let _ = tauri::async_runtime::spawn_blocking(move || session::sweep_stale(&root)).await;
                if tools::status(&bg).ready {
                    hwdetect::ensure(bg).await;
                }
            });

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(SHOW_FALLBACK).await;
                if let Some(w) = handle.get_webview_window("main") {
                    if !w.is_visible().unwrap_or(true) {
                        let _ = w.show();
                    }
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building VideoCrop");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            if let Some(state) = handle.try_state::<AppState>() {
                cleanup(state.inner().clone());
            }
        }
    });
}

fn resolve_paths(app: &tauri::AppHandle) -> Result<Paths, Box<dyn std::error::Error>> {
    let path = app.path();
    Ok(Paths {
        config_dir: path.app_config_dir()?,
        tools_dir: path.app_local_data_dir()?.join("tools"),
        exe_tools_dir: std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("tools"))),
        temp_root: std::env::temp_dir().join("videocrop"),
    })
}

/// Cancels a running encode (deleting its partial output), stops the HLS job and removes the
/// session temp dir. Bounded so a stuck child can't hang exit; the Job Object kills any
/// survivor when the process ends and the next startup sweeps leftovers.
fn cleanup(state: AppState) {
    let encode_done = state::lock(&state.encode).as_ref().map(|h| {
        h.cancel();
        h.finished()
    });
    let session = state.take_session();
    tauri::async_runtime::block_on(async move {
        let work = async {
            if let Some(done) = encode_done {
                done.await;
            }
            if let Some(s) = session {
                s.close().await;
                drop(s); // last reference -> temp dir removed
            }
        };
        let _ = tokio::time::timeout(Duration::from_secs(4), work).await;
    });
}
