use serde::Serialize;
use std::process::Command;

#[derive(Serialize)]
pub struct VideoInfo {
    width: u32,
    height: u32,
    duration: String,
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
        "-v", "error",
        "-select_streams", "v:0",
        "-show_entries", "stream=width,height",
        "-of", "default=noprint_wrappers=1:nokey=1",
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

    let mut output_str = "";
    match std::str::from_utf8(&output.stdout) {
        Ok(str) => {
            output_str = str;
        }
        Err(err) => {
            return Err(err.to_string());
        }
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
