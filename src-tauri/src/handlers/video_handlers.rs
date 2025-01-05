use rfd::FileDialog;

const ALLOWED_VIDEO_EXTENSIONS: [&str; 5] = ["mp4", "avi", "mov", "mkv", "webm"];

#[tauri::command]
pub fn open_video() -> String {
    loop {
        let picked_file = FileDialog::new()
            .add_filter("Video files", &ALLOWED_VIDEO_EXTENSIONS)
            .pick_file();

        match picked_file {
            Some(picked_f) => {
                if check_if_file_is_video(picked_f.to_str().unwrap()) {
                    return picked_f.to_string_lossy().to_string();
                }
            }
            None => {
                return String::from("No file selected");
            }
        }
    }
}

#[tauri::command]
pub fn pick_output_path() -> String {
    println!("Picking output path");
    let file = FileDialog::new()
        .set_title("Select Output Path")
        .pick_folder();

    match file {
        Some(path) => path.to_string_lossy().to_string(),
        None => String::from("No path selected"),
    }
}

fn check_if_file_is_video(file_path: &str) -> bool {
    ALLOWED_VIDEO_EXTENSIONS
        .iter()
        .any(|&ext| file_path.to_lowercase().ends_with(ext))
}
