import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import VideoView from "./components/VideoView";
import { Button, Divider, Switch } from "antd";
import CutSegment from "./components/CutSegment";
import CropSegment from "./components/CropSegment";
import { VideoCropPoints, videoPathIsValid } from "./Utils";
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
        <div className="general-video-options-container">
          <div style={{ width: "20%"}}>
            <div style={{marginBottom: "20px"}}>
              <Button size="large" onClick={get_video_path} type="primary">Select new video</Button>
            </div>
            <CompressSegment disabled={!videoPathIsValid(videoPath)} />
          </div>
          <VideoView videoCropPoints={videoCropPoints} videoPath={videoPath} onVideoPathClick={function (): void {
            get_video_path();
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

}

export default App;
