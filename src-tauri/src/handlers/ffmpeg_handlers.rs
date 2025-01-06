use std::{os::windows::process::CommandExt, process::Command, thread};

use crate::utils::ffmpeg_utils::{
    self, DependenciesSetUpInfo, VideoEditOptions, VideoEditProgress, VideoInfo,
};

pub const CREATE_NO_WINDOW: u32 = 0x08000000;

fn is_command_available(command: &str) -> bool {
    match Command::new(command)
        .creation_flags(CREATE_NO_WINDOW)
        .arg("-version")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                true
            } else {
                println!(
                    "Command was not successful: {}. It returned with error: {:?}",
                    command, output.stderr,
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
    println!("Checking for ffmpeg and ffprobe");
    if is_command_available("ffmpeg") && is_command_available("ffprobe") {
        return true;
    }
    return ffmpeg_utils::add_ffmpeg_to_app_env_if_it_exists();
}

#[tauri::command]
pub fn get_video_info(video_path: &str) -> Result<VideoInfo, String> {
    ffmpeg_utils::get_video_info(video_path)
}

#[tauri::command]
pub fn submit_video_for_editing(options: VideoEditOptions, process_audio: bool) {
    ffmpeg_utils::clear_video_progress();

    thread::spawn(move || {
        ffmpeg_utils::process_video(options.clone(), process_audio);
    });
}

#[tauri::command]
pub fn submit_audio_extraction(options: VideoEditOptions) {
    println!(
        "Extracting audio from video: {:?}",
        options.input_video_path.as_str()
    );
    ffmpeg_utils::clear_video_progress();

    thread::spawn(move || {
        ffmpeg_utils::extract_audio(options.clone());
    });
}

#[tauri::command]
pub fn get_video_progress_info() -> VideoEditProgress {
    return ffmpeg_utils::get_video_progress_info();
}

#[tauri::command]
pub fn download_ffmpeg_windows() {
    thread::spawn(move || {
        ffmpeg_utils::download_and_add_ffmpeg_to_path_windows();
    });
}

#[tauri::command]
pub fn get_depencencies_download_info() -> DependenciesSetUpInfo {
    ffmpeg_utils::get_depencencies_download_info()
}
