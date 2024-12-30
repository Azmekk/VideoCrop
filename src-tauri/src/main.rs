// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Builder, api::path::app_dir};

use std::sync::Mutex;

static FFMPEG_PATH: Mutex<Option<String>> = Mutex::new(None);
static FFPROBE_PATH: Mutex<Option<String>> = Mutex::new(None);

fn main() {
    videocrop_lib::run()
}
