use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn is_command_available(command: &str) -> bool {
    Command::new(command)
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn download_file(url: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url);
    let mut file = fs::File::create(dest)?;
    let content = response?.bytes()?;
    file.write_all(&content)?;
    Ok(())
}

fn download_ffmpeg_windows(){

}

fn download_ffmpeg_linux(){
    
}

#[tauri::command]
pub fn check_ffmpeg_and_ffprobe() -> bool {
    let mut ffmpeg_found = false;
    let mut ffprobe_found = false;

    if is_command_available("ffmpeg") {
        ffmpeg_found = true;
    } else {
        let downloaded_ffmpeg_path = std::env::current_exe().unwrap().parent().unwrap().join("ffmpeg");
        if ffmpeg_path.exists() {
            ffmpeg_found = true;
        }
    }

    let app_directory = app_dir().expect("Failed to get app directory");
    let ffmpeg_path = app_directory.join("ffmpeg");
    let ffprobe_path = app_directory.join("ffprobe");

    if !ffmpeg_path.exists() {
        download_file("URL_TO_FFMPEG", &ffmpeg_path).expect("Failed to download ffmpeg");
    }

    if !ffprobe_path.exists() {
        download_file("URL_TO_FFPROBE", &ffprobe_path).expect("Failed to download ffprobe");
    }

    is_command_available(ffmpeg_path.to_str().unwrap())
        && is_command_available(ffprobe_path.to_str().unwrap())
}
