use crate::commands;
use tauri::{Builder, Runtime};

pub trait BuilderExt<R> {
    fn register_all_handlers(self) -> Self;
}

impl<R: Runtime> BuilderExt<R> for Builder<R> {
    fn register_all_handlers(self) -> Self {
        self.invoke_handler(tauri::generate_handler![
            commands::video_handlers::open_video,
            commands::video_handlers::pick_output_path,
            commands::ffmpeg_handlers::check_ffmpeg_and_ffprobe,
            commands::ffmpeg_handlers::get_video_info,
            commands::ffmpeg_handlers::submit_video_for_editing,
            commands::ffmpeg_handlers::get_video_progress_info
        ])
    }
}
