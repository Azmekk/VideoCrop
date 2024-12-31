use rfd::FileDialog;

#[tauri::command]
pub fn open_video() -> String {
    let file = FileDialog::new()
        .add_filter("Video files", &["mp4", "avi", "mov", "mkv", "webm"])
        .pick_file();

    match file {
        Some(path) => path.to_string_lossy().to_string(),
        None => String::from("No file selected"),
    }
}