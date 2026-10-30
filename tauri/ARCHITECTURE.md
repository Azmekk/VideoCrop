# VideoCrop 2 (Tauri) — architecture

Port of the WinUI 3 app (`../src`, `../VideoCrop_Plan.md`) to Tauri 2 + Svelte 5.
Product goals, in priority order: **small bundle, fast startup**, plays every file
(3-hour HEVC OBS recordings included) with no up-front processing, modern light/dark UI,
feature parity (cut, crop, resize, compress, output modes).

mpv is gone. Playback is `<video>` fed either the original file or an on-demand HLS
stream produced by ffmpeg. Only ffmpeg/ffprobe are downloaded at first run.

```
tauri/
  index.html, vite.config.ts        Svelte 5 + Vite, no UI framework, system font stack
  src/                              frontend
    lib/types.ts                    mirror of core/models.rs (THE contract)
    lib/api.ts                      every invoke/listen lives here
  src-tauri/src/
    core/        pure, deterministic, unit-tested. No processes, no Tauri, no fs writes.
      models.rs  data contract (serde camelCase)
      probe.rs   ffprobe args + JSON -> VideoInfo
      progress.rs  -progress pipe:1 parser
      encode.rs  final-encode argument builder (port of EncodeCommandBuilder.cs)
      naming.rs  _VideoCrop output names (port of OutputNamer.cs)
      presets.rs quality presets (port of PresetLibrary.cs) + platform presets
      planner.rs JobDraft -> JobPlan (single source of truth for the summary)
      preview.rs HLS playlist/job args, frame-grab args, encoder probe, direct-play hint
    media/       runtime: processes, files, network
      process.rs spawn helper: CREATE_NO_WINDOW, kill-on-close Job Object, stdout/stderr lines
      tools.rs   locate + download ffmpeg/ffprobe (BtbN GPL zip) into app-local-data/tools
      toolcheck.rs  PATH ffmpeg: scoop-shim resolution, encoder/version check, cached verdict
      session.rs one open video: id, info, temp dir, HLS job state, frame grabs
      stream.rs  HLS job supervisor (start/restart/throttle/retry)
      protocol.rs `stream` URI scheme handler (direct w/ ranges, playlist, segments, frames)
      encoder.rs runs JobPlan passes, emits progress/done, cancel
      hwdetect.rs picks the preview encoder once, cached in settings
    commands.rs  #[tauri::command] wrappers (thin)
    state.rs     AppState (Mutex'd session/encode/settings)
    settings.rs  settings.json in app config dir
    lib.rs       builder, plugins, protocol registration, window-show fallback
```

## Commands and events (see `src/lib/api.ts`)

| command | args | returns |
|---|---|---|
| `tools_status` | – | `ToolStatus` |
| `tools_download` | – | `ToolStatus` (emits `tools://progress`) |
| `open_video` | `path` | `OpenedVideo` (closes previous session) |
| `close_video` | – | – |
| `presets` | – | `PresetCatalog` |
| `plan_job` | `draft: JobDraft` | `JobPlan` (needs an open video) |
| `encode_start` | `draft` | `JobPlan` (emits `encode://progress` `EncodeStatus`, then `encode://done` `EncodeResult`) |
| `encode_cancel` | – | – |
| `settings_get` / `settings_set` | `settings` | `Settings` / – |
| `reveal_in_folder` | `path` | – (`explorer /select,`) |

Errors are returned as `Err(String)` with a user-readable message.

## The `stream` protocol

Registered with `register_asynchronous_uri_scheme_protocol("stream", …)`. On Windows the
webview sees it as `http://stream.localhost/…`. All paths are scoped by a random session id;
requests for a stale session get 404.

| path | response |
|---|---|
| `/{sid}/direct` | the source file. Honors `Range: bytes=a-b` with 206 + `Content-Range`; responses are capped at 4 MiB per request so huge files are never buffered (the browser asks for the next range). `Accept-Ranges: bytes`. Content-Type from extension. |
| `/{sid}/hls/index.m3u8` | `preview::hls_playlist(duration)` |
| `/{sid}/hls/seg/{n}.ts` | the segment; waits for the HLS job to produce it (see below). `video/mp2t`. |
| `/{sid}/frame?t=&w=&fast=` | one JPEG from `preview::frame_args`, piped from ffmpeg stdout. `w` omitted = native size. |

Every response sets `Access-Control-Allow-Origin: *` and `Cache-Control: no-store` (segments
may use `max-age`).

## Playback strategy

1. **Direct** (zero cost, full quality). `open_video` returns `directLikelyPlayable`
   (core heuristic: container is mp4/mov/webm family, video h264 8-bit 4:2:0 / vp8 / vp9 / av1,
   audio aac/mp3/opus/vorbis/flac or none). The frontend then confirms with
   `video.canPlayType(...)` (HEVC is per-machine) and plays `directUrl`.
2. **Stream** (anything ffmpeg decodes, any length, no wait). If direct is unlikely, or the
   `<video>` fires `error`/stalls on decode, the player switches to `hlsUrl` via a lazily
   imported hls.js at the same timestamp.

### On-demand HLS
- Playlist is synthetic: VOD, `#EXT-X-TARGETDURATION:4`, N = ceil(duration / 4) segments,
  each exactly 4 s except the last, `#EXT-X-ENDLIST`. Nothing is encoded up front.
- One continuous ffmpeg job per session produces `seg_<n>.ts` from `start_segment` onward:
  `-ss <start> -i in -map 0:v:0 -map 0:a:0? -vf scale=-2:min(ih,H) -c:v <enc> <low-latency opts>
  -force_key_frames expr:gte(t,n_forced*4) -c:a aac -ac 2 -b:a 160k -f hls -hls_time 4
  -hls_segment_type mpegts -hls_flags temp_file+independent_segments -start_number <n>
  -hls_list_size 0 -output_ts_offset <start> -hls_segment_filename <dir>/seg_%d.ts <dir>/job.m3u8`.
  A continuous run (not one ffmpeg per segment) avoids audio clicks and timestamp drift.
- Segment request `n`:
  - file exists -> serve.
  - job running and `job.start <= n <= job.next + 4` -> wait for it (poll ~40 ms, 30 s timeout).
  - otherwise -> kill job, restart at `n`, wait.
  - Throttle: kill the job when it is more than 12 segments ahead of the newest request;
    the next request restarts it where needed. Cached segments are reused.
- Encoder: `hwdetect` tries h264_nvenc -> h264_qsv -> h264_amf -> libx264 once with a tiny
  lavfi encode and caches the winner in settings. Height cap 1080 for hardware, 720 for x264.
  `-hwaccel auto` first; if the job dies before producing its first segment, retry once
  without hwaccel, then once with libx264.
- Session temp dir `%TEMP%/videocrop/<sid>/` is deleted on close and on app exit; stale dirs
  from crashed runs are swept at startup.

### Still frames
Used for scrubbing (`fast=1`, `w≈640`, keyframe-snapped, ~50–200 ms anywhere in the file),
the timeline filmstrip (`fast=1`, `w=160`) and the crop editor (`fast=0`, native size, exact
frame). The frontend keeps at most one scrub request in flight and coalesces to the latest.

## Editing rules (unchanged from the WinUI app, see ../CLAUDE.md)
- Filter order fixed: `crop=W:H:X:Y,scale=W2:H2:flags=lanczos`. Crop in source pixels;
  resize sees the post-crop frame; even dimensions; `-2` for the derived axis when locked.
- `-ss` before `-i` = fast cut, after `-i` = accurate cut.
- Compression off => stream copy, crop/resize ignored (summary says so).
- Audio-only => `.m4a/.opus/.mp3/.mka`, crop/resize not applicable.
- Output name `<base>_VideoCrop[N].<ext>`, existing suffix stripped.
- Strip extras: first video + first audio (`-map 0:v:0 -map 0:a:0?`), `-map_metadata -1`.
- Two-pass supports h264/h265/vp9/av1.
- Cut min duration 100 ms; playback constrained to [start, end] and loops.

## Platform presets (planner rules)
| id | name | cap | box | base |
|---|---|---|---|---|
| discord | Discord | 10 MB | 1920×1080 | h264, aac 128k |
| discord-nitro-basic | Discord Nitro Basic | 50 MB | 1920×1080 | h264, aac 128k |
| discord-nitro | Discord Nitro | 500 MB | 2560×1440 | h264, aac 160k |
| youtube | YouTube | – | – | h264 crf 18 slow, aac 320k |
| x | X / Twitter | 512 MB | 1920×1200 | h264, aac 128k |
| vertical | TikTok / Reels / Shorts | – | 1080×1920 | h264 crf 21, aac 128k |
| whatsapp | WhatsApp | 16 MB | 1280×720 | h264, aac 96k |
| email | Email | 25 MB | 1280×720 | h264, aac 96k |

- With a cap: two-pass target-size. Budget bitrate from `compute_target_bitrate(cap, cutDuration, audio, 0.97)`,
  then `min(budget, quality ceiling)` where the ceiling (spec.target_bitrate_bps) stops a 10 s clip
  ballooning to 500 MB (e.g. 8 Mbps @1080p, 12 Mbps @1440p).
- Resolution: if the user set no resize, pick the largest height on the ladder
  [2160,1440,1080,720,540,480,360] that is <= post-crop height, fits the box, and keeps
  bits/pixel/frame >= 0.06 (h264). If nothing qualifies use 360 and warn
  ("Clip is long for 10 MB — consider cutting it"). A user resize wins but is still shrunk to the box.
- Never upscale. Vertical preset warns when the post-crop frame is not portrait
  ("Tip: crop to 9:16").

## Frontend
- Svelte 5 runes, no component library, inline SVG icons, system font
  (`"Segoe UI Variable", "Segoe UI", system-ui`). Lazy `import("hls.js")` only on the stream path.
- Window starts hidden (`visible:false`); the app calls `getCurrentWindow().show()` after first
  paint. Backend shows it anyway after 3 s as a fallback.
- Frameless window with a custom title bar (drag region, theme toggle, min/max/close).
- Theme tokens on `:root`, `[data-theme=dark|light]`, `prefers-color-scheme` for "system";
  saved to localStorage (`vc-theme`, read in index.html before paint) and settings.
- In plain-browser dev (`pnpm dev` without Tauri) `lib/api.ts` callers get a mock backend
  (dev-only dynamic import) so the UI can be iterated in a browser.
- Update check: `check_update` command (GitHub latest release, notification only) + `UpdatePopup.svelte`; skipped version saved as `Settings.skippedUpdate`.
