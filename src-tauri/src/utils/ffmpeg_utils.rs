use serde::{Deserialize, Serialize};
use std::{io::BufRead, process::Command, sync::Mutex};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    width: u32,
    height: u32,
    duration: String,
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
        let new_output_path = output_path.join(new_file_name);
        ffmpeg_args.push(new_output_path.to_str().unwrap().to_string());
    } else {
        let random_guid = Uuid::new_v4().to_string();
        let random_output_path = output_path.join(format!("VideoCrop_{}.mp4", random_guid));
        ffmpeg_args.push(random_output_path.to_str().unwrap().to_string());
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
    println!("Executing command: {}", command_str);

    let mut child = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
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
}

pub fn get_video_progress_info() -> VideoEditProgress {
    let progress = VIDEO_EDIT_PROGRESS.lock().unwrap();
    progress.clone()
}
