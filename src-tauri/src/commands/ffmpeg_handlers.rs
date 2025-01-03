use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::process::Command;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref VIDEO_EDIT_OPTIONS: Mutex<VideoEditOptions> = Mutex::new(VideoEditOptions {
        output_video_path: String::new(),
        cut_options_enabled: false,
        cut_options: None,
        crop_enabled: false,
        crop_options: None,
        compression_enabled: false,
        compression_options: None,
        resize_enabled: false,
        resize_options: None,
    });
}

#[derive(Serialize)]
pub struct VideoInfo {
    width: u32,
    height: u32,
    duration: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VideoEditOptions {
    pub output_video_path: String,
    pub cut_options_enabled: bool,
    pub cut_options: Option<VideoCutOptions>,
    pub crop_enabled: bool,
    pub crop_options: Option<VideoCropPoints>,
    pub compression_enabled: bool,
    pub compression_options: Option<VideoCompressionOptions>,
    pub resize_enabled: bool,
    pub resize_options: Option<ResizeOptions>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VideoCutOptions {
    pub starting_time_string: String,
    pub end_time_string: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VideoCropPoints {
    pub starting_x_offset: i32,
    pub starting_y_offset: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VideoCompressionOptions {
    pub codec: String,
    pub preset: String,
    pub using_crf: bool,
    pub crf: Option<i32>,
    pub bitrate: Option<i32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResizeOptions {
    pub width: i32,
    pub height: i32,
}

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
    let args = [
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=width,height",
        "-of",
        "default=noprint_wrappers=1:nokey=1",
        video_path,
    ];

    let command_str = format!(
        "{} {}",
        "ffprobe",
        args.iter()
            .map(|arg| format!("\"{}\"", arg)) // Quote each argument to handle spaces
            .collect::<Vec<String>>()
            .join(" ")
    );
    println!("Executing command: {}", command_str);

    let output = Command::new("ffprobe")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute ffprobe for width and height: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe failed with error: {}",
            std::str::from_utf8(&output.stderr).unwrap_or("Unknown error")
        ));
    }

    let output_str = match std::str::from_utf8(&output.stdout) {
        Ok(str) => str,
        Err(err) => return Err(err.to_string()),
    };

    let mut lines = output_str.lines();

    let width: u32 = lines
        .next()
        .ok_or("Missing width")?
        .parse()
        .map_err(|e| format!("Failed to parse width: {}", e))?;

    let height: u32 = lines
        .next()
        .ok_or("Missing height")?
        .parse()
        .map_err(|e| format!("Failed to parse height: {}", e))?;

    let duration_output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            "-sexagesimal",
            video_path,
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffprobe for duration: {}", e))?;

    if !duration_output.status.success() {
        return Err(format!(
            "ffprobe for duration failed with error: {}",
            std::str::from_utf8(&duration_output.stderr).unwrap_or("Unknown error")
        ));
    }

    let duration = std::str::from_utf8(&duration_output.stdout)
        .map_err(|e| format!("Failed to parse duration output: {}", e))?
        .trim()
        .to_string();

    Ok(VideoInfo {
        width,
        height,
        duration,
    })
}

#[tauri::command]
pub fn submit_video_for_editing(options: VideoEditOptions) -> Result<String, String> {
    let mut video_edit_options = VIDEO_EDIT_OPTIONS.lock().unwrap();
    *video_edit_options = options;

    let temp_dir = env::temp_dir();
    let file_path = temp_dir.join("video_crop_progress_file.txt");

    match File::create(&file_path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to create or open file: {}", e)),
    };

    Ok(file_path.to_string_lossy().to_string())
}
