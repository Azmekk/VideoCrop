//! `stream` URI scheme (`http://stream.localhost/...` in WebView2). Every request is answered
//! from a tokio task, never on the protocol thread.
//!
//! | path                      | response                                  |
//! |---------------------------|-------------------------------------------|
//! | `/{sid}/direct`           | source file, Range-aware, <= 4 MiB/resp   |
//! | `/{sid}/hls/index.m3u8`   | synthetic VOD playlist                    |
//! | `/{sid}/hls/seg/{n}.ts`   | segment (waits for / restarts the job)    |
//! | `/{sid}/frame?t=&w=&fast=`| JPEG                                      |

use std::io::SeekFrom;
use std::path::Path;
use std::sync::Arc;

use tauri::http::{header, Method, Request, Response, StatusCode};
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::media::session::{FrameError, Session};
use crate::media::stream::{self, SegmentError};
use crate::state::AppState;

pub const SCHEME: &str = "stream";
/// Max bytes in one direct-file response; the media element asks for the next range.
pub const MAX_CHUNK: u64 = 4 * 1024 * 1024;

/// Base URL the webview uses for this scheme.
pub fn base_url() -> &'static str {
    if cfg!(any(windows, target_os = "android")) {
        "http://stream.localhost"
    } else {
        "stream://localhost"
    }
}

pub fn handler<R: Runtime>(ctx: UriSchemeContext<'_, R>, req: Request<Vec<u8>>, responder: UriSchemeResponder) {
    let state = ctx.app_handle().try_state::<AppState>().map(|s| s.inner().clone());
    tauri::async_runtime::spawn(async move {
        let resp = match state {
            Some(state) => {
                let session = parse_route(req.uri().path(), req.uri().query())
                    .and_then(|(sid, _)| state.session_by_id(&sid));
                handle(session, &req).await
            }
            None => plain(StatusCode::SERVICE_UNAVAILABLE, "starting"),
        };
        responder.respond(resp);
    });
}

#[derive(Debug, Clone, PartialEq)]
pub enum Route {
    Direct,
    Playlist,
    Segment(u32),
    Frame { t: f64, w: Option<u32>, fast: bool },
}

/// `/{sid}/...` -> (sid, route).
pub fn parse_route(path: &str, query: Option<&str>) -> Option<(String, Route)> {
    let mut parts = path.trim_start_matches('/').split('/');
    let sid = parts.next().filter(|s| !s.is_empty())?.to_owned();
    let rest: Vec<&str> = parts.collect();
    let route = match rest.as_slice() {
        ["direct"] => Route::Direct,
        ["hls", "index.m3u8"] => Route::Playlist,
        ["hls", "seg", file] => Route::Segment(file.strip_suffix(".ts")?.parse().ok()?),
        ["frame"] => {
            let mut t = 0.0;
            let mut w = None;
            let mut fast = false;
            for pair in query.unwrap_or("").split('&') {
                let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
                match k {
                    "t" => t = v.parse::<f64>().ok().filter(|t| t.is_finite())?,
                    "w" => w = if v.is_empty() { None } else { Some(v.parse::<u32>().ok().filter(|w| *w > 0)?) },
                    "fast" => fast = v == "1" || v.eq_ignore_ascii_case("true"),
                    _ => {}
                }
            }
            Route::Frame { t: t.max(0.0), w, fast }
        }
        _ => return None,
    };
    Some((sid, route))
}

/// Handles one request against the session that owns it (None = unknown/stale sid -> 404).
pub async fn handle(session: Option<Arc<Session>>, req: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let method = req.method();
    if method == Method::OPTIONS {
        return with_common(Response::builder().status(StatusCode::NO_CONTENT))
            .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD, OPTIONS")
            .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "Range")
            .body(Vec::new())
            .unwrap();
    }
    if method != Method::GET && method != Method::HEAD {
        return plain(StatusCode::METHOD_NOT_ALLOWED, "method not allowed");
    }
    let Some((_, route)) = parse_route(req.uri().path(), req.uri().query()) else {
        return plain(StatusCode::NOT_FOUND, "not found");
    };
    let Some(session) = session.filter(|s| !s.is_closed()) else {
        return plain(StatusCode::NOT_FOUND, "no such session");
    };
    let head = method == Method::HEAD;
    match route {
        Route::Direct => {
            let range = req.headers().get(header::RANGE).and_then(|v| v.to_str().ok());
            serve_direct(Path::new(session.input()), range, head).await
        }
        Route::Playlist => {
            let body = (session.fns.hls_playlist)(session.info.duration_sec).into_bytes();
            bytes(StatusCode::OK, "application/vnd.apple.mpegurl", "no-store", body, head)
        }
        Route::Segment(n) => match stream::serve_segment(session, n).await {
            Ok(body) => bytes(StatusCode::OK, "video/mp2t", "max-age=3600", body, head),
            Err(SegmentError::NotFound) => plain(StatusCode::NOT_FOUND, "no such segment"),
            Err(SegmentError::Superseded) => plain(StatusCode::CONFLICT, "superseded"),
            Err(SegmentError::Timeout) => plain(StatusCode::GATEWAY_TIMEOUT, "segment timed out"),
            Err(SegmentError::Failed(e)) => plain(StatusCode::INTERNAL_SERVER_ERROR, &e),
        },
        Route::Frame { t, w, fast } => match session.grab_frame(t, w, fast).await {
            Ok(body) => bytes(StatusCode::OK, "image/jpeg", "no-store", body, head),
            Err(FrameError::Superseded) => plain(StatusCode::CONFLICT, "superseded"),
            Err(FrameError::Closed) => plain(StatusCode::NOT_FOUND, "no such session"),
            Err(FrameError::Failed(e)) => plain(StatusCode::INTERNAL_SERVER_ERROR, &e),
        },
    }
}

// ------------------------------------------------------------------ ranges

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteRange {
    /// No (usable) Range header.
    Full,
    /// Inclusive bounds, already clamped to the file.
    Partial { start: u64, end: u64 },
    Unsatisfiable,
}

/// Parses a `Range` header against a file of `len` bytes. Only the first range of a
/// multi-range request is honored; malformed headers are ignored (RFC 9110 §14.2).
pub fn parse_range(header: Option<&str>, len: u64) -> ByteRange {
    let Some(spec) = header.map(str::trim).and_then(|h| h.strip_prefix("bytes=")) else {
        return ByteRange::Full;
    };
    let first = spec.split(',').next().unwrap_or("").trim();
    let Some((a, b)) = first.split_once('-') else { return ByteRange::Full };
    let (a, b) = (a.trim(), b.trim());
    if a.is_empty() {
        // Suffix range: last `b` bytes.
        let Ok(suffix) = b.parse::<u64>() else { return ByteRange::Full };
        if suffix == 0 || len == 0 {
            return ByteRange::Unsatisfiable;
        }
        return ByteRange::Partial { start: len.saturating_sub(suffix), end: len - 1 };
    }
    let Ok(start) = a.parse::<u64>() else { return ByteRange::Full };
    let end = if b.is_empty() {
        u64::MAX
    } else {
        match b.parse::<u64>() {
            Ok(e) if e >= start => e,
            _ => return ByteRange::Full,
        }
    };
    if start >= len {
        return ByteRange::Unsatisfiable;
    }
    ByteRange::Partial { start, end: end.min(len - 1) }
}

/// What to send: `(status, start, end_inclusive)`; None = 416. Caps at `MAX_CHUNK`.
/// A request without Range for a file larger than the cap is answered as `bytes=0-`.
pub fn plan_direct(range: ByteRange, len: u64) -> Option<(StatusCode, u64, u64)> {
    let (status, start, end) = match range {
        ByteRange::Unsatisfiable => return None,
        ByteRange::Full if len <= MAX_CHUNK => return Some((StatusCode::OK, 0, len.saturating_sub(1))),
        ByteRange::Full => (StatusCode::PARTIAL_CONTENT, 0, len - 1),
        ByteRange::Partial { start, end } => (StatusCode::PARTIAL_CONTENT, start, end),
    };
    Some((status, start, end.min(start + MAX_CHUNK - 1)))
}

pub fn content_type_for(path: &Path) -> &'static str {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        // ISO-BMFF family; Chromium sniffs the container anyway.
        "mp4" | "m4v" | "mov" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "ts" | "m2ts" | "mts" => "video/mp2t",
        "avi" => "video/x-msvideo",
        "ogv" => "video/ogg",
        "m4a" => "audio/mp4",
        "mp3" => "audio/mpeg",
        "opus" | "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        _ => "application/octet-stream",
    }
}

async fn serve_direct(path: &Path, range: Option<&str>, head: bool) -> Response<Vec<u8>> {
    let mut file = match tokio::fs::File::open(path).await {
        Ok(f) => f,
        Err(_) => return plain(StatusCode::NOT_FOUND, "source file missing"),
    };
    let len = match file.metadata().await {
        Ok(m) => m.len(),
        Err(e) => return plain(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };
    let Some((status, start, end)) = plan_direct(parse_range(range, len), len) else {
        return with_common(Response::builder().status(StatusCode::RANGE_NOT_SATISFIABLE))
            .header(header::CACHE_CONTROL, "no-store")
            .header(header::ACCEPT_RANGES, "bytes")
            .header(header::CONTENT_RANGE, format!("bytes */{len}"))
            .body(Vec::new())
            .unwrap();
    };
    let count = if len == 0 { 0 } else { end - start + 1 };
    let mut builder = with_common(Response::builder().status(status))
        .header(header::CONTENT_TYPE, content_type_for(path))
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_LENGTH, count);
    if status == StatusCode::PARTIAL_CONTENT {
        builder = builder.header(header::CONTENT_RANGE, format!("bytes {start}-{end}/{len}"));
    }
    if head || count == 0 {
        return builder.body(Vec::new()).unwrap();
    }
    let mut body = vec![0u8; count as usize];
    let read = async {
        file.seek(SeekFrom::Start(start)).await?;
        file.read_exact(&mut body).await
    };
    if let Err(e) = read.await {
        return plain(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }
    builder.body(body).unwrap()
}

// ------------------------------------------------------------------ response helpers

fn with_common(b: tauri::http::response::Builder) -> tauri::http::response::Builder {
    b.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_EXPOSE_HEADERS, "Content-Range, Content-Length, Accept-Ranges")
}

fn bytes(status: StatusCode, ctype: &str, cache: &str, body: Vec<u8>, head: bool) -> Response<Vec<u8>> {
    let b = with_common(Response::builder().status(status))
        .header(header::CONTENT_TYPE, ctype)
        .header(header::CACHE_CONTROL, cache)
        .header(header::CONTENT_LENGTH, body.len());
    b.body(if head { Vec::new() } else { body }).unwrap()
}

fn plain(status: StatusCode, msg: &str) -> Response<Vec<u8>> {
    bytes(status, "text/plain; charset=utf-8", "no-store", msg.as_bytes().to_vec(), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIB: u64 = 1024 * 1024;

    #[test]
    fn range_forms() {
        assert_eq!(parse_range(None, 100), ByteRange::Full);
        assert_eq!(parse_range(Some("bytes=0-"), 100), ByteRange::Partial { start: 0, end: 99 });
        assert_eq!(parse_range(Some("bytes=10-19"), 100), ByteRange::Partial { start: 10, end: 19 });
        assert_eq!(parse_range(Some("bytes=90-500"), 100), ByteRange::Partial { start: 90, end: 99 });
        assert_eq!(parse_range(Some("bytes=-10"), 100), ByteRange::Partial { start: 90, end: 99 });
        assert_eq!(parse_range(Some("bytes=-500"), 100), ByteRange::Partial { start: 0, end: 99 });
        assert_eq!(parse_range(Some("bytes=5-9, 20-30"), 100), ByteRange::Partial { start: 5, end: 9 });
    }

    #[test]
    fn range_unsatisfiable_and_malformed() {
        assert_eq!(parse_range(Some("bytes=100-"), 100), ByteRange::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=200-300"), 100), ByteRange::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=-0"), 100), ByteRange::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=0-"), 0), ByteRange::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=9-3"), 100), ByteRange::Full);
        assert_eq!(parse_range(Some("items=0-5"), 100), ByteRange::Full);
        assert_eq!(parse_range(Some("bytes=abc"), 100), ByteRange::Full);
    }

    #[test]
    fn direct_plan_caps_chunks() {
        let len = 100 * MIB;
        assert_eq!(
            plan_direct(ByteRange::Partial { start: 0, end: len - 1 }, len),
            Some((StatusCode::PARTIAL_CONTENT, 0, 4 * MIB - 1))
        );
        assert_eq!(
            plan_direct(ByteRange::Partial { start: 10, end: 20 }, len),
            Some((StatusCode::PARTIAL_CONTENT, 10, 20))
        );
        assert_eq!(plan_direct(ByteRange::Full, len), Some((StatusCode::PARTIAL_CONTENT, 0, 4 * MIB - 1)));
        assert_eq!(plan_direct(ByteRange::Full, 1000), Some((StatusCode::OK, 0, 999)));
        assert_eq!(plan_direct(ByteRange::Unsatisfiable, len), None);
    }

    #[test]
    fn routes() {
        assert_eq!(parse_route("/abc/direct", None), Some(("abc".into(), Route::Direct)));
        assert_eq!(parse_route("/abc/hls/index.m3u8", None), Some(("abc".into(), Route::Playlist)));
        assert_eq!(parse_route("/abc/hls/seg/17.ts", None), Some(("abc".into(), Route::Segment(17))));
        assert_eq!(
            parse_route("/abc/frame", Some("t=12.5&w=640&fast=1")),
            Some(("abc".into(), Route::Frame { t: 12.5, w: Some(640), fast: true }))
        );
        assert_eq!(
            parse_route("/abc/frame", Some("t=3")),
            Some(("abc".into(), Route::Frame { t: 3.0, w: None, fast: false }))
        );
        assert_eq!(parse_route("/abc/frame", Some("t=nan")), None);
        assert_eq!(parse_route("/abc/hls/seg/x.ts", None), None);
        assert_eq!(parse_route("/abc/other", None), None);
        assert_eq!(parse_route("/", None), None);
    }

    #[test]
    fn content_types() {
        assert_eq!(content_type_for(Path::new("a.MP4")), "video/mp4");
        assert_eq!(content_type_for(Path::new("a.webm")), "video/webm");
        assert_eq!(content_type_for(Path::new("a.xyz")), "application/octet-stream");
    }
}
