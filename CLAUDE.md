# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Windows desktop app for cutting, cropping, resizing and compressing videos. Tauri 2 (Rust) + Svelte 5, with ffmpeg/ffprobe as external processes. All app code lives in `tauri/`. `tauri/ARCHITECTURE.md` is the spec and the data contract: read it before changing behavior.

Versions up to 2.x were a WinUI 3 / .NET app with mpv playback. That code was removed and is only in git history (before the 3.0 merge).

## Commands

Run from `tauri/` unless noted.

- **Install:** `pnpm install`
- **Run the app:** `pnpm tauri dev`. Set `VIDEOCROP_TOOLS_DIR` to a folder with ffmpeg.exe/ffprobe.exe to override tool lookup.
- **UI only, in a browser:** `pnpm dev`, which uses the mock backend in `src/lib/mock/backend.ts`. URL flags: `?stream` (HLS path), `?tools=0` (setup screen), `?fail` (failed encode), `?update` (update popup). The sample clip lives in gitignored `dev-media/`.
- **Frontend check:** `pnpm check` (svelte-check, must be 0 errors / 0 warnings).
- **Rust tests:** `cargo test` in `tauri/src-tauri`. Single test: `cargo test planner::tests::discord`. Real-ffmpeg smoke test: `VIDEOCROP_TOOLS_DIR=… VIDEOCROP_SMOKE_FILE=… cargo test smoke -- --ignored --nocapture`.
- **Lint:** `cargo clippy --all-targets -- -D warnings` in `tauri/src-tauri`.
- **Installer:** `pnpm tauri build` produces an NSIS installer in `src-tauri/target/release/bundle/nsis`.
- **Website:** `site/` is the static download page, served by Cloudflare Workers via the root `wrangler.jsonc` (Cloudflare's Git integration deploys it on push). `npx wrangler dev` previews it locally. `site/assets/downloads.js` fills download links from the latest GitHub release, matching `VideoCrop-vX.Y.Z-{x64,arm64}-{setup,portable}.exe`; keep those asset names in sync with `release.yml`.
- **Release:** push a `vX.Y.Z` tag. `.github/workflows/release.yml` builds x64 (`windows-latest`) and arm64 (`windows-11-arm`, native) and publishes the installer + portable exe. `workflow_dispatch` builds artifacts without publishing.

## Architecture

- **`src-tauri/src/core/`** is pure logic: no processes, no Tauri, no filesystem writes (existence checks are injected closures). It covers ffmpeg argument building (`encode.rs`, `preview.rs`), ffprobe parsing, progress parsing, output naming, presets and the planner. Everything here is unit-tested; keep it that way.
- **`core/models.rs` is the IPC contract** (serde camelCase) and is mirrored by `src/lib/types.ts`. Change both together.
- **`src-tauri/src/media/`** is the runtime. It spawns processes (`process.rs`: `CREATE_NO_WINDOW`, kill-on-close Job Object), resolves and downloads tools (`tools.rs`, `toolcheck.rs`), serves the `stream` protocol (`protocol.rs`), supervises the on-demand HLS preview (`stream.rs`, `session.rs`), runs encodes (`encoder.rs`) and detects hardware encoders (`hwdetect.rs`).
- **`commands.rs`** holds thin `#[tauri::command]` wrappers.
- **Frontend:** `src/lib/api.ts` is the only place that calls `invoke`/`listen`. State lives in rune stores (`src/lib/state/*.svelte.ts`), pure helpers in `src/lib/util/`, and components in `src/components/`. The editor is lazy-loaded and hls.js is lazy-imported only on the stream path.

### Playback

`<video>` plays the original file (`/{sid}/direct`, range requests) when WebView2 can decode it. Otherwise it plays an on-demand HLS stream (`/{sid}/hls/…`): a synthetic VOD playlist of 4 s segments, produced by one continuous ffmpeg job that restarts on far seeks and is throttled when it runs ahead. Still frames (`/{sid}/frame`) drive scrubbing, the filmstrip and the crop editor. Details are in ARCHITECTURE.md.

### Rules that must not regress

- Filter chain order is fixed: `crop=W:H:X:Y,scale=W2:H2:flags=lanczos`. `CropSpec` is in **source pixels**. Resize sees the **post-crop** frame. Output dimensions are even, using `-2` for the derived axis when aspect-locked.
- `-ss` before `-i` gives a fast, keyframe-snapped cut; after `-i` gives an accurate one (`CutSpec.accurate`).
- With compression off, everything is stream-copied and crop/resize are ignored. Audio-only output goes to `.m4a/.opus/.mp3/.mka`.
- Output names are `<base>_VideoCrop[N].<ext>`, with an existing suffix stripped first.
- The planner's `JobPlan.summary` is the single source of truth shown before Process. Never compute summaries in TypeScript.
- Progress comes from `-progress pipe:1 -nostats` via `ProgressParser`. Never scrape stderr for progress.
- Arguments are always passed as a `Vec<String>`, never as a shell string.
- Respect anamorphic SAR (`VideoInfo::display_aspect_ratio`).
- When changing `core/encode.rs` or `core/planner.rs`, extend their tests. Bad command construction is the most common source of bugs in apps like this.

### Defaults to confirm with the user before changing

- `_VideoCrop` suffix.
- Stream-copy audio on cut-only.
- Single-file processing.
- Container tied to codec, with an override in Custom.
- Keep only the first video + first audio stream.
- Windows 10 1809+ baseline.
- Platform preset caps (Discord 10 MB, etc.).

## Product goals

Small bundle and fast startup are explicit priorities: keep dependencies minimal on both sides, keep the main JS chunk lean, and do nothing slow in Tauri `setup`. Updates are notification-only (`check_update` + `UpdatePopup.svelte`); there is deliberately no auto-installer.
