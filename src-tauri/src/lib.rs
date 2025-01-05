mod handlers;
mod utils;

//use extensions::builder_extensions::BuilderExt;
use tauri_plugin_updater::UpdaterExt;

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
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                println!("Sleeping update thread...");
                tokio::time::sleep(tokio::time::Duration::from_secs(600)).await;
                update(handle).await.unwrap();
            });
            Ok(())
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app
        .updater_builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?
        .check()
        .await?
    {
        let mut downloaded = 0;

        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    println!("downloaded {downloaded} from {content_length:?}");
                },
                || {
                    println!("download finished");
                },
            )
            .await?;

        println!("update installed");
        app.restart();
    }

    Ok(())
}
