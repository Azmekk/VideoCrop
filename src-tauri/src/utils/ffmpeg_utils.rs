use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::fs::File;
use std::os::windows::process::CommandExt;
use std::{io::BufRead, path::PathBuf, process::Command, sync::Mutex};
use uuid::Uuid;

pub const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    width: u32,
    height: u32,
    duration: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DependenciesSetUpInfo {
    status: String,
    completed: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VideoEditProgress {
    progress: f64,
    working: bool,
    last_error: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VideoEditOptions {
    pub input_video_path: String,
    pub output_video_path: String,
    pub cut_options_enabled: bool,
    pub cut_options: VideoCutOptions,
    pub crop_enabled: bool,
    pub crop_options: VideoCropPoints,
    pub compression_enabled: bool,
    pub compression_options: VideoCompressionOptions,
    pub resize_enabled: bool,
    pub resize_options: ResizeOptions,
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
    pub crf: i32,
    pub bitrate: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResizeOptions {
    pub width: i32,
    pub height: i32,
}

lazy_static::lazy_static! {
    static ref VIDEO_EDIT_PROGRESS: Mutex<VideoEditProgress> = Mutex::new(VideoEditProgress {
        progress: 0.0,
        working: false,
        last_error: None,
    });

    static ref FFMPEG_DOWNLOAD_PROGRESS: Mutex<DependenciesSetUpInfo> = Mutex::new(DependenciesSetUpInfo {
        status: "".to_string(),
        completed: false,
    });
}

fn convert_time_string_to_seconds(time_string: &str) -> Result<f64, String> {
    let parts: Vec<&str> = time_string.split(':').collect();
    if parts.len() != 3 {
        return Err("Invalid time format".to_string());
    }

    let hours: f64 = parts[0]
        .parse()
        .map_err(|e| format!("Failed to parse hours: {}", e))?;
    let minutes: f64 = parts[1]
        .parse()
        .map_err(|e| format!("Failed to parse minutes: {}", e))?;
    let seconds: f64 = parts[2]
        .parse()
        .map_err(|e| format!("Failed to parse seconds: {}", e))?;

    Ok(hours * 3600.0 + minutes * 60.0 + seconds)
}

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
        &video_path,
    ];

    let command_str = format!(
        "{} {}",
        "ffprobe",
        args.iter()
            .map(|arg| format!("\"{}\"", arg))
            .collect::<Vec<String>>()
            .join(" ")
    );
    //println("Executing command: {}", command_str);

    let output = Command::new("ffprobe")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
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
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            "-sexagesimal",
            &video_path,
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

pub fn get_video_length_in_seconds(video_path: &str) -> Result<f64, String> {
    let duration_output = Command::new("ffprobe")
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            &video_path,
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
        .parse::<f64>()
        .unwrap();
    Ok(duration)
}

pub fn process_video(options: VideoEditOptions) {
    let mut progress = VIDEO_EDIT_PROGRESS.lock().unwrap();
    progress.working = true;
    progress.progress = 0.0;
    progress.last_error = None;
    drop(progress);

    let mut ffmpeg_args = vec!["-i".to_string(), options.input_video_path.clone()];

    let mut video_length: f64 = get_video_length_in_seconds(&options.input_video_path).unwrap();

    if options.cut_options_enabled {
        let cut_options = &options.cut_options;
        ffmpeg_args
            .extend_from_slice(&["-ss".to_string(), cut_options.starting_time_string.clone()]);
        ffmpeg_args.extend_from_slice(&["-to".to_string(), cut_options.end_time_string.clone()]);

        let start_time_seconds =
            convert_time_string_to_seconds(&cut_options.starting_time_string).unwrap_or(0.0);
        let end_time_seconds =
            convert_time_string_to_seconds(&cut_options.end_time_string).unwrap_or(video_length);
        video_length = end_time_seconds - start_time_seconds;
    }

    let video_length_in_us = (video_length * 1_000_000.0) as u64;

    if options.crop_enabled {
        let crop_options = &options.crop_options;
        let crop_filter = format!(
            "crop={}:{}:{}:{}",
            crop_options.width,
            crop_options.height,
            crop_options.starting_x_offset,
            crop_options.starting_y_offset
        );
        ffmpeg_args.extend_from_slice(&["-vf".to_string(), crop_filter]);
    }

    if options.compression_enabled {
        let compression_options = &options.compression_options;
        ffmpeg_args.extend_from_slice(&["-c:v".to_string(), compression_options.codec.clone()]);
        ffmpeg_args.extend_from_slice(&["-preset".to_string(), compression_options.preset.clone()]);

        if compression_options.using_crf {
            ffmpeg_args
                .extend_from_slice(&["-crf".to_string(), compression_options.crf.to_string()]);
        } else {
            ffmpeg_args
                .extend_from_slice(&["-b:v".to_string(), compression_options.bitrate.to_string()]);
        }
    }

    if options.resize_enabled && !options.crop_enabled {
        let resize_options = &options.resize_options;
        ffmpeg_args.extend_from_slice(&[
            "-vf".to_string(),
            format!("scale={}:{}", resize_options.width, resize_options.height),
        ]);
    }

    ffmpeg_args.extend_from_slice(&[
        "-progress".to_string(),
        "-".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
    ]);

    let input_path = std::path::Path::new(&options.input_video_path);
    let output_path = std::path::Path::new(&options.output_video_path);

    if let Some(file_stem) = input_path.file_stem() {
        let new_file_name = format!(
            "{}_VideoCrop.{}",
            file_stem.to_string_lossy(),
            input_path.extension().unwrap().to_str().unwrap()
        );

        let new_output_path = get_unique_filename(&output_path.join(new_file_name));
        ffmpeg_args.push(new_output_path);
    } else {
        let random_guid = Uuid::new_v4().to_string();
        let random_output_path =
            get_unique_filename(&output_path.join(format!("VideoCrop_{}.mp4", random_guid)));

        ffmpeg_args.push(random_output_path);
    }

    let command_str = format!(
        "{} {}",
        "ffmpeg",
        ffmpeg_args
            .iter()
            .map(|arg| format!("\"{}\"", arg))
            .collect::<Vec<String>>()
            .join(" ")
    );
    //println("Executing command: {}", command_str);

    let mut child = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .expect("Failed to start ffmpeg process");

    let stdout = child.stdout.take().expect("Failed to capture stdout");

    let stdout_reader = std::io::BufReader::new(stdout);

    let stdout_thread = std::thread::spawn(move || {
        for line in stdout_reader.lines() {
            if let Ok(line) = line {
                if line.starts_with("out_time_us=") {
                    if let Some(value) = line.split('=').nth(1) {
                        if let Ok(out_time_us) = value.parse::<u64>() {
                            let mut progress = VIDEO_EDIT_PROGRESS.lock().unwrap();
                            progress.progress =
                                ((out_time_us as f64 / video_length_in_us as f64) * 100.0).round();

                            if progress.progress >= 100.0 {
                                progress.progress = 99.0;
                            }
                            drop(progress);
                        }
                    }
                }
            }
        }
    });

    let status = child.wait().expect("Failed to wait on ffmpeg process");

    stdout_thread.join().expect("Failed to join stdout thread");

    if !status.success() {
        eprintln!("ffmpeg process failed with status: {}", status.clone());

        let mut progress = VIDEO_EDIT_PROGRESS.lock().unwrap();
        progress.working = false;
        progress.progress = 0.0;
        progress.last_error = Some(status.to_string());
        drop(progress);
    }

    let mut progress = VIDEO_EDIT_PROGRESS.lock().unwrap();
    if progress.last_error.is_none() {
        progress.working = false;
        progress.progress = 100.0;
        progress.last_error = None;
    }
    drop(progress);

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        clear_video_progress();
    });
}

fn get_unique_filename(path: &PathBuf) -> String {
    let mut unique_path = path.to_path_buf();
    let mut counter = 1;

    while unique_path.exists() {
        let file_stem = path.file_stem().unwrap().to_string_lossy();
        let extension = path.extension().unwrap_or_default().to_string_lossy();
        let new_file_name = format!("{}_{}.{}", file_stem, counter, extension);
        unique_path = path.with_file_name(new_file_name);
        counter += 1;
    }

    unique_path.to_string_lossy().to_string()
}

pub fn clear_video_progress() {
    let mut progress = VIDEO_EDIT_PROGRESS.lock().unwrap();
    progress.progress = 0.0;
    progress.working = false;
    progress.last_error = None;
    drop(progress);
}

pub fn get_video_progress_info() -> VideoEditProgress {
    let progress = VIDEO_EDIT_PROGRESS.lock().unwrap();
    progress.clone()
}

pub fn update_ffmpeg_download_status(status: &str, completed: bool) {
    let mut progress = FFMPEG_DOWNLOAD_PROGRESS.lock().unwrap();
    progress.status = status.to_string();
    progress.completed = completed;
    drop(progress);
}

pub fn get_depencencies_download_info() -> DependenciesSetUpInfo {
    let progress = FFMPEG_DOWNLOAD_PROGRESS.lock().unwrap();
    progress.clone()
}

pub fn download_and_add_ffmpeg_to_path_windows() {
    if !cfg!(target_os = "windows") {
        panic!("This function is only supported on Windows.");
    }

    let user_path_var = env::var("USERPROFILE").unwrap();
    let temp_path_var = env::var("TEMP").unwrap();

    let user_path = std::path::Path::new(&user_path_var);
    let temp_path = std::path::Path::new(&temp_path_var);

    let video_crop_ffmpeg_path = user_path.join("VideoCrop").join("FFmpeg");

    if !video_crop_ffmpeg_path.exists() {
        fs::create_dir_all(&video_crop_ffmpeg_path).unwrap();
    }

    update_ffmpeg_download_status("Downloading...", false);

    let download_url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip";
    let ffmpeg_zip_download_path = &temp_path.join(format!(
        "ffmpeg-master-latest-win64-gpl_VideoCrop_{}.zip",
        Uuid::new_v4().to_string()
    ));
    let extract_path = &video_crop_ffmpeg_path.join("ffmpeg-master-latest-win64-gpl_VideoCrop");

    if extract_path.exists() {
        fs::remove_dir_all(extract_path).unwrap();
    }

    let ffmpeg_bin_path = extract_path.join("bin");
    if ffmpeg_bin_path.exists() {
        add_ffmpeg_to_app_env(ffmpeg_bin_path.to_str().unwrap());
        return;
    }

    let download_result = reqwest::blocking::get(download_url);
    match download_result {
        Ok(mut response) => {
            let mut file = fs::File::create(ffmpeg_zip_download_path).unwrap();
            response.copy_to(&mut file).unwrap();
        }
        Err(e) => {
            panic!("Failed to download ffmpeg: {}", e);
        }
    }

    //println("Extracting ffmpeg to: {:?}", extract_path);

    update_ffmpeg_download_status("Extracting...", false);
    let zip_cursor = File::open(&ffmpeg_zip_download_path).unwrap();
    zip_extract::extract(zip_cursor, &extract_path, true).unwrap();

    add_ffmpeg_to_app_env(ffmpeg_bin_path.to_str().unwrap());

    update_ffmpeg_download_status("Finalizing...", true);
    fs::remove_file(ffmpeg_zip_download_path).unwrap();
}

//Cool code that nuked my $PATH value earlier LOL
//DO NOT UNCOMMENT SIMPLY KEEPING THIS AS A FUNNY MEMORY

//pub fn add_path_to_path_env_if_not_added(new_path: &str) {
//    if !cfg!(target_os = "windows") {
//        panic!("This function is only supported on Windows.");
//    }
//
//    let reg_key = RegKey::predef(HKEY_CURRENT_USER)
//        .open_subkey_with_flags("Environment", KEY_ALL_ACCESS)
//        .unwrap();
//
//    let user_path: String = reg_key.get_value("Path").unwrap();
//
//    if !user_path.contains(new_path) {
//        let updated_path = format!("{};{}", user_path, new_path);
//        reg_key.set_value("Path", &updated_path).unwrap();
//        //println("Updating user path to: {}", updated_path);
//    } else {
//        //println("Path already exists in the user PATH.");
//    }
//}

pub fn add_ffmpeg_to_app_env(new_path: &str) {
    if !cfg!(target_os = "windows") {
        panic!("This function is only supported on Windows.");
    }

    let mut current_path = env::var("PATH").unwrap_or_default();

    if !current_path.ends_with(";") {
        current_path.push(';');
    }

    if !current_path.contains(new_path) {
        let updated_path = format!("{}{}", current_path, new_path);
        env::set_var("PATH", updated_path.clone());
        //println("Updating app path to: {}", updated_path);
    } else {
        //println("Path already exists in the app PATH.");
    }
}

pub fn add_ffmpeg_to_app_env_if_it_exists() -> bool {
    let user_path_var = env::var("USERPROFILE").unwrap();

    let user_path = std::path::Path::new(&user_path_var);

    let video_crop_ffmpeg_path = user_path.join("VideoCrop").join("FFmpeg");

    let extract_path = &video_crop_ffmpeg_path.join("ffmpeg-master-latest-win64-gpl_VideoCrop");

    let ffmpeg_bin_path = extract_path.join("bin");
    //println("Checking for dependencies at: {:?}", ffmpeg_bin_path);
    if ffmpeg_bin_path.exists() {
        //println("Found dependencies at: {:?}", ffmpeg_bin_path);
        add_ffmpeg_to_app_env(ffmpeg_bin_path.to_str().unwrap());
        return true;
    }

    false
}
