# VideoCrop

A small Windows app for trimming, cropping, resizing, and compressing video files. Pick a clip, tweak what you want, hit Process.

## Features

- **Cut** a precise range with a draggable timeline (frame-accurate or fast keyframe-snapped).
- **Crop** to any rectangle or snap to common aspects (16:9, 4:3, 1:1, 9:16, original).
- **Resize** with aspect lock and one-click presets (2160p / 1440p / 1080p / 720p / 480p).
- **Compress** with sensible quality presets (H.264 for compatibility, H.265 / AV1 for smaller files) plus an advanced panel for custom bitrate or target file size.
- **Output mode** — keep everything, or extract just audio (`.m4a` / `.opus` / `.mp3`) or just video.
- Embedded preview with transport + volume controls. Auto-updates from new releases.

## Download

Grab the latest `VideoCrop-vX.Y.Z-win-x64.zip` (or `win-arm64.zip`) from the [Releases page](https://github.com/Azmekk/VideoCrop/releases). Unzip and run `VideoCrop.App.exe` — no installer.

## Where to put it

Recommended: `%LOCALAPPDATA%\Programs\VideoCrop` (i.e. `C:\Users\<you>\AppData\Local\Programs\VideoCrop`). Avoid `Program Files` — auto-updates need to write to the install folder without UAC prompts.

## Requirements

- Windows 10 version 1809 (October 2018) or newer, 64-bit.
- [.NET 10 Desktop Runtime](https://dotnet.microsoft.com/download/dotnet/10.0) (the installer will prompt if it's missing).
- [Windows App SDK 1.6+ runtime](https://learn.microsoft.com/windows/apps/windows-app-sdk/downloads).

## First run

VideoCrop uses ffmpeg and mpv under the hood. The first time you launch the app, it'll offer to download them (~200 MB total) into a `tools/` folder next to the exe. Click **Download** and wait — that's it.

## Using it

1. Drag a video onto the window (or click **Open File…**). It plays back in the preview pane.
2. Adjust any combination of:
   - **Cut** — start/end times, or drag the handles on the timeline.
   - **Crop** — click **Edit crop…**, drag a rectangle, optionally pick an aspect, hit Apply.
   - **Resize** — toggle on, pick a preset height or type a custom size.
   - **Compression** — pick a preset; flip on *Custom (advanced)* if you need codec-level control.
3. Confirm the output destination, then click **Process**.

The output filename is the original with `_VideoCrop` appended (collisions get `_VideoCrop2`, `_VideoCrop3`, etc.).

## Keyboard shortcuts

| Key | Action |
|---|---|
| `Ctrl + O` | Open file |
| `Space` | Play / pause |
| `I` | Set cut start to current playhead |
| `O` | Set cut end to current playhead |
| `Ctrl + Enter` | Process |

## Updates

VideoCrop checks GitHub for a new release each time it starts. When one is available, the in-app banner downloads it in the background and offers a **Restart and update** button — the swap and relaunch happen automatically.

## Credits

- [ffmpeg / ffprobe](https://ffmpeg.org/) — the encoding workhorses.
- [mpv](https://mpv.io/) — playback and frame screenshots, used as a separate process under its GPLv2+ license.
