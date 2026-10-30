//! End-to-end smoke test of the `stream` protocol against a real ffmpeg and file. Ignored by
//! default. Run with:
//! `VIDEOCROP_TOOLS_DIR=<dir with ffmpeg.exe+ffprobe.exe> VIDEOCROP_SMOKE_FILE=<video> cargo test smoke -- --ignored --nocapture`

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use tauri::http::{header, Request};

use crate::core::models::VideoInfo;
use crate::core::preview::PreviewEncoder;
use crate::core::probe;
use crate::media::protocol::{handle, MAX_CHUNK};
use crate::media::session::{PreviewFns, Session};
use crate::media::stream::MAX_AHEAD;
use crate::media::process;

fn req(method: &str, uri: &str, range: Option<&str>) -> Request<Vec<u8>> {
    let mut b = Request::builder().method(method).uri(format!("http://stream.localhost{uri}"));
    if let Some(r) = range {
        b = b.header(header::RANGE, r);
    }
    b.body(Vec::new()).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn smoke_protocol_end_to_end() {
    let tools = PathBuf::from(std::env::var("VIDEOCROP_TOOLS_DIR").expect("VIDEOCROP_TOOLS_DIR"));
    let file = std::env::var("VIDEOCROP_SMOKE_FILE").expect("VIDEOCROP_SMOKE_FILE");
    let len = std::fs::metadata(&file).unwrap().len();
    let out = process::run_capture(&tools.join("ffprobe.exe"), &probe::ffprobe_args(&file), Some(Duration::from_secs(30)))
        .await
        .unwrap();
    assert!(out.status.success(), "{}", out.stderr);
    let info = probe::parse_ffprobe_json(&String::from_utf8_lossy(&out.stdout), &file, Some(len)).unwrap();
    run_smoke(PreviewFns::default(), info, tools.join("ffmpeg.exe")).await;
}

/// The protocol walk-through, shared so it can also run with alternative `PreviewFns`.
pub async fn run_smoke(fns: PreviewFns, info: VideoInfo, ffmpeg: PathBuf) {
    let len = info.file_size_bytes.unwrap();
    let duration = info.duration_sec;

    let root = std::env::temp_dir().join("videocrop-smoke");
    let enc = Arc::new(RwLock::new(Some(PreviewEncoder::X264)));
    let s = Session::create(info, ffmpeg, &root, enc, fns).unwrap();
    let sid = s.id.clone();
    let dir = s.dir.clone();
    let get = |uri: String, range: Option<&'static str>| {
        let s = s.clone();
        async move { handle(Some(s), &req("GET", &uri, range)).await }
    };

    // Direct ranges.
    let r = get(format!("/{sid}/direct"), Some("bytes=0-")).await;
    assert_eq!(r.status(), 206);
    assert_eq!(r.body().len() as u64, MAX_CHUNK.min(len));
    assert_eq!(r.headers()[header::CONTENT_RANGE], format!("bytes 0-{}/{len}", MAX_CHUNK.min(len) - 1));
    let r = get(format!("/{sid}/direct"), Some("bytes=-100")).await;
    assert_eq!((r.status().as_u16(), r.body().len()), (206, 100));
    let r = get(format!("/{sid}/direct"), Some("bytes=999999999999-")).await;
    assert_eq!(r.status(), 416);
    let r = handle(Some(s.clone()), &req("HEAD", &format!("/{sid}/direct"), Some("bytes=0-"))).await;
    assert_eq!(r.status(), 206);
    assert!(r.body().is_empty());
    let r = handle(None, &req("GET", &format!("/{sid}/direct"), None)).await;
    assert_eq!(r.status(), 404);

    // Playlist.
    let r = get(format!("/{sid}/hls/index.m3u8"), None).await;
    assert_eq!(r.status(), 200);

    // Frames.
    for (q, label) in [("t=60&w=640&fast=1", "scrub"), ("t=30&w=160&fast=1", "filmstrip"), ("t=45.5&fast=0", "exact")] {
        let t0 = Instant::now();
        let r = get(format!("/{sid}/frame?{q}"), None).await;
        assert_eq!(r.status(), 200, "{label}: {}", String::from_utf8_lossy(r.body()));
        assert_eq!(&r.body()[..2], &[0xFF, 0xD8]);
        println!("{label} frame {} bytes in {:?}", r.body().len(), t0.elapsed());
    }
    // Coalescing: two scrubs back to back -> the first is superseded (409), the second wins.
    let first = tokio::spawn(get(format!("/{sid}/frame?t=10&w=640&fast=1"), None));
    tokio::time::sleep(Duration::from_millis(20)).await;
    let second = get(format!("/{sid}/frame?t=20&w=640&fast=1"), None).await;
    let first = first.await.unwrap();
    println!("coalesce: first={} second={}", first.status(), second.status());
    assert_eq!(second.status(), 200);

    // HLS: first segments, a far seek (restart), then back (restart again).
    let seg_count = (fns.segment_count)(duration);
    for n in [0u32, 1, 2, 20, 21, 5] {
        let t0 = Instant::now();
        let r = get(format!("/{sid}/hls/seg/{n}.ts"), None).await;
        assert_eq!(r.status(), 200, "seg {n}: {}", String::from_utf8_lossy(r.body()));
        assert_eq!(r.body()[0], 0x47, "mpegts sync byte");
        println!("seg {n}: {} bytes in {:?}", r.body().len(), t0.elapsed());
    }
    let r = get(format!("/{sid}/hls/seg/{}.ts", seg_count), None).await;
    assert_eq!(r.status(), 404);

    // Throttle: with seg 5 the newest request, the job must stop around 5 + MAX_AHEAD.
    tokio::time::sleep(Duration::from_secs(8)).await;
    let on_disk: Vec<u32> = (0..seg_count).filter(|n| dir.join(format!("seg_{n}.ts")).exists()).collect();
    println!("segments on disk: {on_disk:?}");
    assert!(on_disk.iter().filter(|n| **n > 5 && **n < 20).all(|n| *n <= 5 + MAX_AHEAD + 2));

    let last = seg_count - 1;
    let t0 = Instant::now();
    let r = get(format!("/{sid}/hls/seg/{last}.ts"), None).await;
    assert_eq!(r.status(), 200);
    println!("last seg {last}: {} bytes in {:?}", r.body().len(), t0.elapsed());

    s.close().await;
    drop(s);
    assert!(!dir.exists(), "session dir removed on last drop");
}
