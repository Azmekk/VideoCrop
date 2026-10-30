//! ffprobe invocation + JSON parsing into `VideoInfo`.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer};

use super::models::*;

pub fn ffprobe_args(path: &str) -> Vec<String> {
    ["-v", "error", "-print_format", "json", "-show_format", "-show_streams", path]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[derive(Debug, Default, Deserialize)]
struct FfprobeOutput {
    #[serde(default)]
    streams: Vec<FfprobeStream>,
    #[serde(default)]
    format: FfprobeFormat,
}

#[derive(Debug, Default, Deserialize)]
struct FfprobeStream {
    #[serde(default)]
    index: u32,
    codec_name: Option<String>,
    codec_type: Option<String>,
    profile: Option<String>,
    pix_fmt: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    sample_aspect_ratio: Option<String>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
    #[serde(default, deserialize_with = "str_or_num")]
    bit_rate: Option<String>,
    channels: Option<u32>,
    #[serde(default, deserialize_with = "str_or_num")]
    sample_rate: Option<String>,
    tags: Option<HashMap<String, serde_json::Value>>,
    #[serde(default, deserialize_with = "str_or_num")]
    duration: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct FfprobeFormat {
    #[serde(default, deserialize_with = "str_or_num")]
    duration: Option<String>,
    #[serde(default, deserialize_with = "str_or_num")]
    bit_rate: Option<String>,
    format_name: Option<String>,
    #[serde(default, deserialize_with = "str_or_num")]
    size: Option<String>,
}

/// ffprobe emits most numbers as strings; accept either.
fn str_or_num<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let v = Option::<serde_json::Value>::deserialize(d)?;
    Ok(match v {
        Some(serde_json::Value::String(s)) => Some(s),
        Some(serde_json::Value::Number(n)) => Some(n.to_string()),
        _ => None,
    })
}

pub fn parse_ffprobe_json(json: &str, path: &str, file_size_bytes: Option<u64>) -> Result<VideoInfo, String> {
    let parsed: FfprobeOutput =
        serde_json::from_str(json).map_err(|e| format!("ffprobe returned unparseable JSON: {e}"))?;

    let is_type = |s: &FfprobeStream, t: &str| s.codec_type.as_deref().is_some_and(|c| c.eq_ignore_ascii_case(t));

    let video = parsed
        .streams
        .iter()
        .find(|s| is_type(s, "video"))
        .ok_or_else(|| "Input has no video stream.".to_string())?;

    let duration_sec = parse_seconds(parsed.format.duration.as_deref())
        .or_else(|| parse_seconds(video.duration.as_deref()))
        .unwrap_or(0.0);
    let (sar_num, sar_den) = parse_ratio(video.sample_aspect_ratio.as_deref()).unwrap_or((1, 1));
    let fps = parse_rate(video.avg_frame_rate.as_deref())
        .filter(|f| *f > 0.0)
        .or_else(|| parse_rate(video.r_frame_rate.as_deref()))
        .unwrap_or(0.0);

    let audio_streams = parsed
        .streams
        .iter()
        .filter(|s| is_type(s, "audio"))
        .map(|s| AudioStreamInfo {
            index: s.index,
            codec: s.codec_name.clone().unwrap_or_else(|| "?".into()),
            channels: s.channels,
            sample_rate: s.sample_rate.as_deref().and_then(|v| v.trim().parse().ok()),
            bitrate_bps: parse_u64(s.bit_rate.as_deref()),
            language: s.tags.as_ref().and_then(|t| t.get("language")).and_then(|v| v.as_str()).map(str::to_string),
        })
        .collect();

    Ok(VideoInfo {
        path: path.to_string(),
        duration_sec,
        width: video.width.unwrap_or(0),
        height: video.height.unwrap_or(0),
        sar_num,
        sar_den,
        video_codec: video.codec_name.clone().unwrap_or_else(|| "?".into()),
        profile: video.profile.clone().filter(|p| !p.is_empty()),
        pix_fmt: video.pix_fmt.clone().filter(|p| !p.is_empty()),
        fps,
        bitrate_bps: parse_u64(parsed.format.bit_rate.as_deref()),
        audio_streams,
        container_format: parsed.format.format_name.clone().unwrap_or_default(),
        file_size_bytes: file_size_bytes.or_else(|| parse_u64(parsed.format.size.as_deref())),
    })
}

fn parse_seconds(raw: Option<&str>) -> Option<f64> {
    raw?.trim().parse::<f64>().ok().filter(|v| v.is_finite() && *v >= 0.0)
}

fn parse_u64(raw: Option<&str>) -> Option<u64> {
    raw?.trim().parse::<u64>().ok()
}

/// `"4:3"` -> (4, 3). A zero denominator or garbage yields None (caller falls back to 1:1).
fn parse_ratio(raw: Option<&str>) -> Option<(u32, u32)> {
    let (n, d) = raw?.split_once(':')?;
    let n: u32 = n.trim().parse().ok()?;
    let d: u32 = d.trim().parse().ok()?;
    if d == 0 {
        return None;
    }
    Some((n, d))
}

/// `"30000/1001"` or `"24"`.
fn parse_rate(raw: Option<&str>) -> Option<f64> {
    let raw = raw?.trim();
    match raw.split_once('/') {
        None => raw.parse::<f64>().ok().filter(|v| v.is_finite()),
        Some((n, d)) => {
            let n: f64 = n.trim().parse().ok()?;
            let d: f64 = d.trim().parse().ok()?;
            if d == 0.0 {
                return None;
            }
            Some(n / d).filter(|v| v.is_finite())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_H264: &str = r#"
    {
      "streams": [
        {
          "index": 0,
          "codec_name": "h264",
          "codec_type": "video",
          "profile": "High",
          "pix_fmt": "yuv420p",
          "width": 1920,
          "height": 1080,
          "sample_aspect_ratio": "1:1",
          "display_aspect_ratio": "16:9",
          "r_frame_rate": "30000/1001",
          "avg_frame_rate": "30000/1001",
          "duration": "12.345000"
        },
        {
          "index": 1,
          "codec_name": "aac",
          "codec_type": "audio",
          "channels": 2,
          "sample_rate": "48000",
          "bit_rate": "192000",
          "tags": { "language": "eng" }
        }
      ],
      "format": {
        "filename": "test.mp4",
        "duration": "12.345000",
        "bit_rate": "5000000",
        "size": "7715625",
        "format_name": "mov,mp4,m4a,3gp,3g2,mj2"
      }
    }
    "#;

    #[test]
    fn args() {
        assert_eq!(
            ffprobe_args("C:/a b/c.mp4"),
            ["-v", "error", "-print_format", "json", "-show_format", "-show_streams", "C:/a b/c.mp4"]
        );
    }

    #[test]
    fn parses_basic_h264_video() {
        let info = parse_ffprobe_json(SAMPLE_H264, "C:/test.mp4", None).unwrap();
        assert_eq!(info.path, "C:/test.mp4");
        assert_eq!(info.width, 1920);
        assert_eq!(info.height, 1080);
        assert_eq!(info.video_codec, "h264");
        assert_eq!(info.profile.as_deref(), Some("High"));
        assert_eq!(info.pix_fmt.as_deref(), Some("yuv420p"));
        assert_eq!((info.sar_num, info.sar_den), (1, 1));
        assert!((info.fps - 29.97).abs() < 0.01);
        assert!((info.duration_sec - 12.345).abs() < 0.001);
        assert_eq!(info.bitrate_bps, Some(5_000_000));
        assert_eq!(info.audio_streams.len(), 1);
        let a = &info.audio_streams[0];
        assert_eq!(a.codec, "aac");
        assert_eq!(a.index, 1);
        assert_eq!(a.channels, Some(2));
        assert_eq!(a.language.as_deref(), Some("eng"));
        assert_eq!(a.sample_rate, Some(48000));
        assert_eq!(a.bitrate_bps, Some(192_000));
        assert!((info.display_aspect_ratio() - 16.0 / 9.0).abs() < 0.01);
        assert_eq!(info.container_format, "mov,mp4,m4a,3gp,3g2,mj2");
        assert_eq!(info.file_size_bytes, Some(7_715_625));
    }

    #[test]
    fn explicit_file_size_wins() {
        let info = parse_ffprobe_json(SAMPLE_H264, "C:/test.mp4", Some(42)).unwrap();
        assert_eq!(info.file_size_bytes, Some(42));
    }

    #[test]
    fn anamorphic_sar_affects_display_aspect_ratio() {
        let json = SAMPLE_H264.replace(r#""sample_aspect_ratio": "1:1""#, r#""sample_aspect_ratio": "4:3""#);
        let info = parse_ffprobe_json(&json, "C:/anamorphic.mp4", None).unwrap();
        assert_eq!((info.sar_num, info.sar_den), (4, 3));
        assert!((info.display_aspect_ratio() - 1920.0 * 4.0 / 3.0 / 1080.0).abs() < 0.01);
    }

    #[test]
    fn bad_sar_falls_back_to_square() {
        let json = SAMPLE_H264.replace(r#""sample_aspect_ratio": "1:1""#, r#""sample_aspect_ratio": "1:0""#);
        let info = parse_ffprobe_json(&json, "C:/x.mp4", None).unwrap();
        assert_eq!((info.sar_num, info.sar_den), (1, 1));
    }

    #[test]
    fn missing_video_stream_errors() {
        let audio_only = r#"
        {
          "streams": [
            { "index": 0, "codec_name": "aac", "codec_type": "audio", "channels": 2, "sample_rate": "44100" }
          ],
          "format": { "duration": "5.0", "format_name": "mp3" }
        }"#;
        assert!(parse_ffprobe_json(audio_only, "C:/x.mp3", None).is_err());
    }

    #[test]
    fn empty_audio_streams_handled() {
        let video_only = r#"
        {
          "streams": [
            { "index": 0, "codec_name": "vp9", "codec_type": "video", "width": 1280, "height": 720, "r_frame_rate": "24/1" }
          ],
          "format": { "duration": "3.0", "format_name": "matroska,webm" }
        }"#;
        let info = parse_ffprobe_json(video_only, "C:/silent.webm", None).unwrap();
        assert!(info.audio_streams.is_empty());
        assert_eq!(info.video_codec, "vp9");
        assert!((info.fps - 24.0).abs() < 0.01);
        assert_eq!(info.profile, None);
        assert_eq!(info.pix_fmt, None);
        assert_eq!(info.bitrate_bps, None);
        assert_eq!(info.file_size_bytes, None);
    }

    #[test]
    fn invalid_json_errors() {
        assert!(parse_ffprobe_json("not json", "x", None).is_err());
        assert!(parse_ffprobe_json("{}", "x", None).is_err());
    }

    #[test]
    fn avg_frame_rate_zero_falls_back_to_r_frame_rate() {
        let json = r#"{"streams":[{"index":0,"codec_type":"video","codec_name":"hevc","width":2560,"height":1440,
            "avg_frame_rate":"0/0","r_frame_rate":"60/1","profile":"Main 10","pix_fmt":"yuv420p10le"}],
            "format":{"format_name":"matroska,webm"}}"#;
        let info = parse_ffprobe_json(json, "C:/obs.mkv", None).unwrap();
        assert_eq!(info.fps, 60.0);
        assert_eq!(info.duration_sec, 0.0);
        assert_eq!(info.profile.as_deref(), Some("Main 10"));
        assert_eq!(info.pix_fmt.as_deref(), Some("yuv420p10le"));
    }

    #[test]
    fn stream_duration_used_when_format_missing_or_na() {
        let json = r#"{"streams":[{"index":0,"codec_type":"video","codec_name":"h264","width":640,"height":480,
            "duration":"7.5"}],"format":{"duration":"N/A","format_name":"mpegts"}}"#;
        let info = parse_ffprobe_json(json, "C:/a.ts", None).unwrap();
        assert_eq!(info.duration_sec, 7.5);
    }

    #[test]
    fn numeric_fields_accepted_as_numbers_and_multiple_audio() {
        let json = r#"{"streams":[
            {"index":0,"codec_type":"video","codec_name":"h264","width":640,"height":480},
            {"index":1,"codec_type":"audio","codec_name":"opus","sample_rate":48000,"bit_rate":96000,"tags":{"language":"jpn","title":"x"}},
            {"index":2,"codec_type":"audio","codec_name":"aac"},
            {"index":3,"codec_type":"subtitle","codec_name":"mov_text"}],
            "format":{"duration":3.25,"bit_rate":800000,"format_name":"mov,mp4"}}"#;
        let info = parse_ffprobe_json(json, "C:/a.mp4", None).unwrap();
        assert_eq!(info.duration_sec, 3.25);
        assert_eq!(info.bitrate_bps, Some(800_000));
        assert_eq!(info.audio_streams.len(), 2);
        assert_eq!(info.audio_streams[0].sample_rate, Some(48000));
        assert_eq!(info.audio_streams[0].bitrate_bps, Some(96_000));
        assert_eq!(info.audio_streams[0].language.as_deref(), Some("jpn"));
        assert_eq!(info.audio_streams[1].index, 2);
        assert_eq!(info.audio_streams[1].language, None);
    }

    #[test]
    fn video_stream_need_not_be_first() {
        let json = r#"{"streams":[
            {"index":0,"codec_type":"audio","codec_name":"aac"},
            {"index":1,"codec_type":"VIDEO","codec_name":"av1","width":100,"height":50,"r_frame_rate":"25"}],
            "format":{"format_name":"mp4"}}"#;
        let info = parse_ffprobe_json(json, "C:/a.mp4", None).unwrap();
        assert_eq!(info.video_codec, "av1");
        assert_eq!(info.fps, 25.0);
        assert_eq!(info.audio_streams.len(), 1);
    }
}
