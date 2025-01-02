import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import VideoView from "./components/VideoView";
import { Button } from "antd";
import CutSegment from "./components/CutSegment";
import CropSegment from "./components/CropSegment";
import type { VideoCropPoints, VideoEditOptions, VideoInfo } from "./Logic/Interfaces";
import "./App.css";
import CompressSegment from "./components/CompressSegment";
import ResizeSegment from "./components/ResizeSegment";
import { initiateVideoCropPoints, videoPathIsValid } from "./Logic/Utils";
import { CropPointsContext } from "./Logic/GlobalContexts";

function App() {
  const [ffmpegExsts, setFfmpegExsts] = useState(true);
  const [videoPath, setVideoPath] = useState("");
  const [videoInfo, setVideoInfo] = useState<VideoInfo | undefined>(undefined);
  const [resetCropPoints, setResetCropPoints] = useState(0);

  const [cropPointPositions, setCropPointPositions] = useState<VideoCropPoints>(initiateVideoCropPoints());

  const [videoEditOptions, setvideoEditOptions] = useState<VideoEditOptions>({
    cutOptionsEnabled: false,
    cutOptions: undefined,
    cropLinesEnabled: false,
    cropPointsEnabled: false,
    cropPoints: undefined,
    compressionEnabled: false,
    compressionOptions: undefined,
    resizeEnabled: false,
    resizeOptions: undefined,
  });

  const videoViewRef = useRef<HTMLVideoElement>(null);

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

    setCropPointPositions({
      startingXOffset: 0,
      startingYOffset: 0,
      width: vidInfo.width,
      height: vidInfo.height,
    });

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
    );
  }

  return (
    <main className="app-container">
      <div className="general-video-options-container">
        <div style={{ width: "20%", display: "flex", flexDirection: "column", gap: "20px" }}>
          <div style={{ marginBottom: "20px" }}>
            <Button size="large" onClick={getVideoPath} type="primary">
              Select new video
            </Button>
          </div>
          <CompressSegment disabled={!videoPathIsValid(videoPath)} />
          <ResizeSegment disabled={!videoPathIsValid(videoPath)} videoInfo={videoInfo} />
        </div>
        <CropPointsContext.Provider value={{ cropPointPositions, setCropPointPositions }}>
          <VideoView
            videoInfo={videoInfo}
            videoPath={videoPath}
            onVideoPathClick={(): void => {
              getVideoPath();
            }}
            resizerEnabled={videoEditOptions.cropLinesEnabled && videoEditOptions.cropPointsEnabled}
            reset={resetCropPoints}
            cropEnabled={videoEditOptions.cropPointsEnabled}
          />
        </CropPointsContext.Provider>
        <div style={{ width: "20%", display: "flex", flexDirection: "column", alignItems: "end" }}>
          <div>
            <CropPointsContext.Provider value={{ cropPointPositions, setCropPointPositions }}>
              <CropSegment
                videoInfo={videoInfo}
                onCropLinesEnabledChanged={(e) => setvideoEditOptions({ ...videoEditOptions, cropLinesEnabled: e })}
                onSegmentEnabledChanged={(e) => setvideoEditOptions({ ...videoEditOptions, cropPointsEnabled: e })}
                disabled={!videoPathIsValid(videoPath)}
                onReset={() => {
                  setResetCropPoints(resetCropPoints + 1);
                  setCropPointPositions({
                    startingXOffset: 0,
                    startingYOffset: 0,
                    width: videoInfo?.width ?? 0,
                    height: videoInfo?.height ?? 0,
                  });
                }}
              />
            </CropPointsContext.Provider>
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
