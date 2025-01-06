<div align="center">

 <p align="center">
  <img src="https://github.com/Azmekk/VideoCrop/blob/master/src-tauri/icons/128x128.png" alt="VideoCrop">
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
[![License][license-shield-GPL3]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]

[![Tauri][tauri-shield]][tauri-url]
[![React][react-shield]][react-url]
[![Rust][rust-shield]][rust-url]
[![TypeScript][TypeScript]][TypeScript-url]

<p align="center">
<img src="https://github.com/Azmekk/VideoCrop/blob/master/assets/VbjPsAYEJP.png" alt="App Example">
</p>

</div>

# VideoCrop

### **VideoCrop** is a desktop application designed to simplify video cutting, editing, cropping, compressing, and audio extraction. It prioritizes simplicity and ease of use over an extensive feature set.

# How to use

1. **Download `VideoCrop_x.x.x_x64_en-US.msi` from [latest](https://github.com/Azmekk/VideoCrop/releases/latest/) release**
2. **Install the app**
3. **Allow the app to install FFMPEG** if you don't already have it. This is essential for video processing.

### Before you start:

<img src="https://github.com/Azmekk/VideoCrop/blob/master/assets/3TJzID8090.png" alt="Select Video">
1. **Select a video**: Choose a video by clicking on the video container or the top left corner 
<p> </p>
<img src="https://github.com/Azmekk/VideoCrop/blob/master/assets/JZ16sWFv5J.png" alt="Select Output">
2. **Select an output path**: Specify the output path from the top right corner.

### Compression
<img src="https://github.com/Azmekk/VideoCrop/blob/master/assets/eZwGf2DpIy.png" alt="Compression Settings">

### Video Codecs

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

### Resize (Only works if cropping is disabled)
<img src="https://github.com/Azmekk/VideoCrop/blob/master/assets/GOlhlrwnhP.png" alt="Resize Settings">

- Adjust the **width** and **height** of the video.
 

### Cropping
<img src="https://github.com/Azmekk/VideoCrop/blob/master/assets/jvbynCFKti.png" alt="Crop Settings">

- **Unlock/Lock** the padlock to enable/disable dragging respectively.
- **Drag and adjust** the crop field to your desired area.
- **Lock** the crop field to interact with the video.
- Note that resizing does not work with cropping.

 <img src="https://github.com/Azmekk/VideoCrop/blob/master/assets/haU09uzu0g.png" alt="No crop and resize">

### Cutting
<img src="https://github.com/Azmekk/VideoCrop/blob/master/assets/S2HWExyKX8.png" alt="Crop Settings">

- Drag to select the portion of the video you want to cut. You can also use optional inputs for precision.

### Submitting 

- **Optional** you can choose to output video or audio only by clicking on the little arrow on the right of the submission button.
- **Submit** by clicking on the actual button to the left of the arrow to start the process of editing your video.
<img src="https://github.com/Azmekk/VideoCrop/blob/master/assets/znzpHFzhue.png" alt="Submission options">
<img src="https://github.com/Azmekk/VideoCrop/blob/master/assets/pmGZKP9ofm.png" alt="Submission">

## Important Note. All of these modifications will only be applied if they have been enabled.

[contributors-shield]: https://img.shields.io/github/contributors/Azmekk/VideoCrop.svg?style=for-the-badge
[contributors-url]: https://github.com/Azmekk/VideoCrop/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/Azmekk/VideoCrop.svg?style=for-the-badge
[forks-url]: https://github.com/Azmekk/VideoCrop/network/members
[stars-shield]: https://img.shields.io/github/stars/Azmekk/VideoCrop.svg?style=for-the-badge
[stars-url]: https://github.com/Azmekk/VideoCrop/stargazers
[issues-shield]: https://img.shields.io/github/issues/Azmekk/VideoCrop.svg?style=for-the-badge
[issues-url]: https://github.com/Azmekk/VideoCrop/issues
[license-shield-GPL3]: https://img.shields.io/github/license/Azmekk/VideoCrop.svg?style=for-the-badge
[license-url]: https://github.com/Azmekk/VideoCrop/blob/master/LICENSE
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
