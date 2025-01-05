mod handlers;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            handlers::video_handlers::open_video,
            handlers::video_handlers::pick_output_path,
            handlers::ffmpeg_handlers::check_ffmpeg_and_ffprobe,
            handlers::ffmpeg_handlers::get_video_info,
            handlers::ffmpeg_handlers::submit_video_for_editing,
            handlers::ffmpeg_handlers::get_video_progress_info,
            handlers::ffmpeg_handlers::download_ffmpeg_windows,
            handlers::ffmpeg_handlers::get_depencencies_download_info,
            handlers::ffmpeg_handlers::submit_audio_extraction,
            handlers::updates_handler::check_for_updates,
            handlers::updates_handler::update_app
        ])
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
