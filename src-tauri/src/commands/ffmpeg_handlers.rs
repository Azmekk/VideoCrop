use std::{f32::consts::E, process::Command};

fn is_command_available(command: &str) -> bool {
    match Command::new(command).arg("-version").output() {
        Ok(output) => {
            if  output.status.success() {
                true
            }
            else{
                println!("Command was not successful: {}. It returned with error: {:?}", command, output.stderr);
                false
            }
        },
        Err(_) => {
            println!("Error checking for command: {}", command);
            false
        },
    }
}

#[tauri::command]
pub fn check_ffmpeg_and_ffprobe() -> bool {
    return is_command_available("ffmpeg") && is_command_available("ffprobe");
}
