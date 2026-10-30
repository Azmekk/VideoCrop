//! ffprobe invocation + JSON parsing into `VideoInfo`.

use super::models::*;

pub fn ffprobe_args(path: &str) -> Vec<String> {
    let _ = path;
    todo!("core agent")
}

pub fn parse_ffprobe_json(json: &str, path: &str, file_size_bytes: Option<u64>) -> Result<VideoInfo, String> {
    let _ = (json, path, file_size_bytes);
    todo!("core agent")
}
