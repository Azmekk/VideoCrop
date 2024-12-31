import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import VideoView from "./components/VideoView";
import { Button, Divider } from "antd";
import CutSegment from "./components/CutSegment";
import { videoPathIsValid } from "./App";
import "./App.css";

interface VideoInfo {
  width: number,
  height: number,
  duration: string,
}
function App() {
  const [ffmpegExsts, setFfmpegExsts] = useState(true);
  const [videoPath, setVideoPath] = useState("");
  const [videoInfo, setVideoInfo] = useState<VideoInfo | undefined>(undefined);
  async function checkFfmpegAndFfprobe() {
    setFfmpegExsts(await invoke("check_ffmpeg_and_ffprobe"));
  }

  let video_selector_open: boolean = false;
  async function get_video_path() {
    if (video_selector_open === true) {
      return;
    }
    else {
      video_selector_open = true;
    }
    let path: string = await invoke("open_video");
    if (path !== "") {
      setVideoPath(path);
    }

    let vidInfo: VideoInfo = await invoke("get_video_info", { videoPath: path });
    setVideoInfo(vidInfo);

    console.log(vidInfo);
    video_selector_open = false;
  }

  async function OnStart() {
    checkFfmpegAndFfprobe();
  }
  useEffect(() => { OnStart() }, []);

  if (!ffmpegExsts) {
    return (
      <div className="ffmpeg-not-downloaded">
        FFmpeg and FFprobe were not located. Please download them and add them to path.
      </div>
    )
  }
  else {
    return (
      <main className="app-container">
        <div className="video-view-container">
          <VideoView videoPath={videoPath} onVideoPathClick={function (): void {
            get_video_path();
          }} />
        </div>

        <Button onClick={get_video_path} type="primary">Select video</Button>
        {videoPathIsValid(videoPath) &&
          <CutSegment videoPath={videoPath} videoDuration={videoInfo?.duration ?? "0:00:00.000"} />
        }
        
      </main>
    );
  }

}

export default App;
