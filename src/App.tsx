import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import VideoView from "./components/VideoView";
import { Button, Divider, Switch } from "antd";
import CutSegment from "./components/CutSegment";
import CropSegment from "./components/CropSegment";
import { type VideoCropPoints, videoPathIsValid } from "./Utils";
import "./App.css";
import CompressSegment from "./components/CompressSegment";

interface VideoInfo {
  width: number,
  height: number,
  duration: string,
}
function App() {
  const [ffmpegExsts, setFfmpegExsts] = useState(true);
  const [videoPath, setVideoPath] = useState("");
  const [videoInfo, setVideoInfo] = useState<VideoInfo | undefined>(undefined);
  const [videoCropPoints, setVideoCropPoints] = useState<VideoCropPoints>({ left: 0, right: 0, bottom: 0, top: 0 });

  let video_selector_open = false;
  async function getVideoPath() {
    if (video_selector_open === true) {
      return;
    }
    
    video_selector_open = true;
    const path: string = await invoke("open_video");
    if (!videoPathIsValid(path)) {
      video_selector_open = false;
      return;
    }

    setVideoPath(path);

    const vidInfo: VideoInfo = await invoke("get_video_info", { videoPath: path });
    setVideoInfo(vidInfo);

    console.log(vidInfo);
  }

  async function checkFfmpegAndFfprobe() {
    setFfmpegExsts(await invoke("check_ffmpeg_and_ffprobe"));
  }

  useEffect(() => {
    checkFfmpegAndFfprobe();
  }, []);

  if (!ffmpegExsts) {
    return (
      <div className="ffmpeg-not-downloaded">
        FFmpeg and FFprobe were not located. Please download them and add them to path.
      </div>
    )
  }
  
    return (
      <main className="app-container">
        <div className="general-video-options-container">
          <div style={{ width: "20%"}}>
            <div style={{marginBottom: "20px"}}>
              <Button size="large" onClick={getVideoPath} type="primary">Select new video</Button>
            </div>
            <CompressSegment disabled={!videoPathIsValid(videoPath)} />
          </div>
          <VideoView videoCropPoints={videoCropPoints} videoPath={videoPath} onVideoPathClick={(): void => {
            getVideoPath();
          }} />
          <div style={{ width: "20%", display: "flex", flexDirection: "column", alignItems: "end" }}>
            <div>
              <Button disabled={true} type="primary">Placeholder</Button>
              <CropSegment videoCropPoints={videoCropPoints} />
            </div>
          </div>

        </div>

        <div className={videoPathIsValid(videoPath) ? "" : "disabled"}>
          <CutSegment videoPath={videoPath} videoDuration={videoInfo?.duration ?? "0:00:00.000"} />
        </div>

      </main>
    );

}

export default App;
