# VideoCrop

A small Windows app for trimming, cropping, resizing, and compressing video files. Pick a clip, tweak what you want, hit Process.

## Features

- **Cut** a precise range with a draggable timeline (frame-accurate or fast keyframe-snapped).
- **Crop** to any rectangle or snap to common aspects (16:9, 4:3, 1:1, 9:16, original).
- **Resize** with aspect lock and one-click presets (2160p / 1440p / 1080p / 720p / 480p).
- **Compress** with sensible quality presets (H.264 for compatibility, H.265 / AV1 for smaller files) plus an advanced panel for custom bitrate or target file size.
- Live preview in a dedicated mpv window. Auto-updates from new releases.

## Download

Grab the latest `VideoCrop-vX.Y.Z-win-x64.zip` (or `win-arm64.zip`) from the [Releases page](https://github.com/Azmekk/VideoCrop/releases). Unzip anywhere — Documents, Desktop, a USB stick — and run `VideoCrop.App.exe`. No installer.

## Requirements

- Windows 10 version 1809 (October 2018) or newer, 64-bit.
- [.NET 10 Desktop Runtime](https://dotnet.microsoft.com/download/dotnet/10.0) (the installer will prompt if it's missing).
- [Windows App SDK 1.6+ runtime](https://learn.microsoft.com/windows/apps/windows-app-sdk/downloads).

## First run

VideoCrop calls out to three small command-line tools — ffmpeg, ffprobe, and mpv. After unzipping, open the `tools/` folder next to `VideoCrop.App.exe` and either:

- Run `fetch-tools.ps1` from PowerShell to download them automatically, **or**
- Drop your own `ffmpeg.exe`, `ffprobe.exe`, and `mpv.exe` into that `tools/` folder.

If the app finds the tools on your system `PATH`, those work too.

## Using it

1. Drag a video onto the window (or click **Open File…**). mpv opens in its own window for preview.
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
