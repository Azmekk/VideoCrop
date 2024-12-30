use tauri::{Builder, Runtime};
use crate::commands;

pub trait BuilderExt<R> {
    fn register_all_handlers(self) -> Self;
}

impl<R: Runtime> BuilderExt<R> for Builder<R> {
    fn register_all_handlers(self) -> Self {
        self
        .invoke_handler(tauri::generate_handler![commands::ffmpeg_handlers::check_ffmpeg_and_ffprobe, commands::video_handlers::open_video])
    }
}