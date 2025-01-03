use std::{process::Command, thread};

use crate::utils::ffmpeg_utils::{self, VideoEditOptions, VideoEditProgress, VideoInfo};

fn is_command_available(command: &str) -> bool {
    match Command::new(command).arg("-version").output() {
        Ok(output) => {
            if output.status.success() {
                true
            } else {
                println!(
                    "Command was not successful: {}. It returned with error: {:?}",
                    command, output.stderr
                );
                false
            }
        }
        Err(_) => {
            println!("Error checking for command: {}", command);
            false
        }
    }
}

#[tauri::command]
pub fn check_ffmpeg_and_ffprobe() -> bool {
    return is_command_available("ffmpeg") && is_command_available("ffprobe");
}

#[tauri::command]
pub fn get_video_info(video_path: &str) -> Result<VideoInfo, String> {
    ffmpeg_utils::get_video_info(video_path)
}

#[tauri::command]
pub fn submit_video_for_editing(options: VideoEditOptions) {
    thread::spawn(move || {
        ffmpeg_utils::process_video(options.clone());
    });
}

#[tauri::command]
pub fn get_video_progress_info() -> VideoEditProgress {
    return ffmpeg_utils::get_video_progress_info();
}
