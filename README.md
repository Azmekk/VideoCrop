# VideoCrop

A small Windows app for trimming, cropping, resizing, and compressing video files. Pick a clip, tweak what you want, hit Process.

## Features

- **Cut** a precise range on a zoomable timeline with a filmstrip: frame-accurate or fast (keyframe-snapped). Drag away from the track while scrubbing to slow down for fine control, or type exact times.
- **Crop** on a full-resolution still to any rectangle, or snap to common aspects (16:9, 4:3, 1:1, 9:16, 4:5, original).
- **Resize** with aspect lock, height presets that never upscale past the original, and percentage chips.
- **Share presets**: one click for Discord, YouTube, X, TikTok/Reels/Shorts, WhatsApp or email. Size caps and output resolution are worked out for you.
- **Compress** with quality presets (H.264 for compatibility, H.265 / AV1 for smaller files), plus a custom panel for codec, bitrate or target file size.
- **Output mode**: keep everything, or extract just the audio (`.m4a` / `.opus` / `.mp3`) or just the video.
- Plays anything ffmpeg can open, including multi-hour HEVC/AV1 OBS recordings, with no up-front conversion. Files Windows can't decode are streamed through an on-demand preview.
- Light and dark themes. Tells you when a new version is out.

## Download

Grab the latest release from the [Releases page](https://github.com/Azmekk/VideoCrop/releases):

- `VideoCrop-vX.Y.Z-x64-setup.exe`: installer (~2 MB, per-user, no admin rights needed, adds a Start menu entry and an uninstaller).
- `VideoCrop-vX.Y.Z-x64-portable.exe`: a single exe you can run from anywhere.
- `-arm64` variants for Windows on ARM.

## Requirements

- Windows 10 version 1809 (October 2018) or newer, 64-bit.
- Microsoft Edge WebView2 runtime. It's built into Windows 11; on Windows 10 the installer fetches it if it's missing.

## First run

VideoCrop uses ffmpeg under the hood. If you already have ffmpeg 6 or newer on your PATH, it is used as is. It needs to be a full GPL build with libx264, libx265, SVT-AV1, VP9, Opus and LAME, like the gyan.dev or BtbN builds.

Otherwise, the first time you launch the app it offers to download one (~100 MB, once) into `%LOCALAPPDATA%\com.azmek.videocrop\tools`. If your PATH ffmpeg didn't qualify, it tells you why.

## Using it

1. Drag a video onto the window (or click to choose a file). It plays in the preview pane.
2. Adjust any combination of:
   - **Cut**: drag the handles on the timeline (mouse wheel zooms), or type start/end times.
   - **Crop**: click **Crop video**, drag the rectangle, optionally pick an aspect, hit Apply.
   - **Resize**: toggle it on, then pick a preset or type a size.
   - **Export**: pick a **Share** target, a **Quality** preset, or go **Custom**.
3. Check the summary line, then click **Process**.

The output filename is the original with `_VideoCrop` appended (collisions get `_VideoCrop2`, `_VideoCrop3`, etc.).

## Keyboard shortcuts

| Key | Action |
|---|---|
| `Ctrl + O` | Open file |
| `Space` | Play / pause |
| `I` / `O` | Set cut start / end to the playhead |
| `←` / `→` | Step one frame |
| `Shift + ←` / `→` | Jump one second |
| `Home` / `End` | Go to cut start / end |
| `+` / `-` | Zoom the timeline |
| `Ctrl + Enter` | Process |

## Updates

A few seconds after starting, VideoCrop checks GitHub for a new release. If there is one, a small popup links to the release page. Download the new installer whenever you like, or skip that version.

## Building

```
cd tauri
pnpm install
pnpm tauri dev      # run with hot reload
pnpm tauri build    # installer in src-tauri/target/release/bundle/nsis
```

Push a `vX.Y.Z` tag to have GitHub Actions build and publish a release.

## Credits

- [ffmpeg / ffprobe](https://ffmpeg.org/): the encoding workhorses.
- [Tauri](https://tauri.app/), [Svelte](https://svelte.dev/) and [hls.js](https://github.com/video-dev/hls.js): the app shell, UI and streaming preview.
