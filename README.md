<div align="center">

 <p align="center">
  <img src="https://github.com/Azmekk/VideoCrop/blob/master/src-tauri/icons/128x128.png" alt="YT-DLP web app">
</p>

  <p align="center">
    <a href="https://github.com/Azmekk/VideoCrop/issues">Report Bug</a>
    ·
    <a href="https://github.com/Azmekk/VideoCrop/issues">Request Feature</a>
  </p>
</div>

<div align="center">
  
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]

[![Tauri][tauri-shield]][tauri-url]
[![React][react-shield]][react-url]
[![Rust][rust-shield]][rust-url]
[![TypeScript][TypeScript]][TypeScript-url]

</div>

# VideoCrop

### **VideoCrop** is a desktop application designed to simplify video cutting, editing, cropping, compressing, and audio extraction. It prioritizes simplicity and ease of use over an extensive feature set.

# How to use

1. **Download latest release**: Ensure you have the latest version of the app.
2. **Allow the app to install FFMPEG** if you don't already have it. This is essential for video processing.

### Before you start:

1. **Select a video**: Choose a video from the top left corner.
2. **Select an output path**: Specify the output path from the top right corner.

### Compression

#### Video Codecs

- **libx264 (H.264)**
  - **Pros**: Widely supported, good balance of quality and file size.
  - **Cons**: Not as efficient as newer codecs.
- **libx265 (H.265)**
  - **Pros**: Better compression, smaller file sizes for the same quality.
  - **Cons**: Slower encoding, less widely supported.
- **AV1**
  - **Pros**: Best compression efficiency, royalty-free.
  - **Cons**: Very slow encoding, limited hardware support.

#### Presets

- **Slow -> Fast**
  - **Slow Presets**: Better compression (smaller file size), higher quality, but very slow encoding.
  - **Fast Presets**: Faster encoding, but larger file size and potentially lower quality.

#### Audio Codecs

- **Opus**
  - **Pros**: High quality at low bitrates, versatile.
  - **Cons**: Not as widely supported as AAC or MP3.
- **AAC**
  - **Pros**: Good quality, widely supported.
  - **Cons**: Slightly larger file sizes compared to Opus.
- **MP3**
  - **Pros**: Very widely supported.
  - **Cons**: Lower quality at the same bitrate compared to Opus and AAC.

#### Best Default Values

- **Video Codec**: `libx264`
- **Preset**: `medium`
- **Audio Codec**: `AAC`

### Resize

- Adjust the **width** and **height** of the video. Note that resizing does not work with cropping.

### Cropping
- **Unlock** Use the padlock to enable dragging.
- **Drag and adjust** the crop field to your desired area.
- **Lock** the crop field to interact with the video.

### Cutting (Only works if cropping is disabled)

- Drag to select the portion of the video you want to cut. You can also use optional inputs for precision.

# Important Note. All of these modifications will only be applied if they have been enabled.

[contributors-shield]: https://img.shields.io/github/contributors/Azmekk/VideoCrop.svg?style=for-the-badge
[contributors-url]: https://github.com/Azmekk/VideoCrop/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/Azmekk/VideoCrop.svg?style=for-the-badge
[forks-url]: https://github.com/Azmekk/VideoCrop/network/members
[stars-shield]: https://img.shields.io/github/stars/Azmekk/VideoCrop.svg?style=for-the-badge
[stars-url]: https://github.com/Azmekk/VideoCrop/stargazers
[issues-shield]: https://img.shields.io/github/issues/Azmekk/VideoCrop.svg?style=for-the-badge
[issues-url]: https://github.com/Azmekk/VideoCrop/issues
[license-shield]: https://img.shields.io/github/license/Azmekk/VideoCrop.svg?style=for-the-badge
[license-url]: https://github.com/Azmekk/VideoCrop/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/Martin-Y
[TypeScript]: https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white
[TypeScript-url]: https://www.typescriptlang.org/
[tauri-shield]: https://img.shields.io/badge/tauri-FFC131?style=for-the-badge&logo=tauri&logoColor=white
[tauri-url]: https://tauri.studio/
[react-shield]: https://img.shields.io/badge/react-61DAFB?style=for-the-badge&logo=react&logoColor=black
[react-url]: https://reactjs.org/
[rust-shield]: https://img.shields.io/badge/rust-000000?style=for-the-badge&logo=rust&logoColor=white
[rust-url]: https://www.rust-lang.org/
