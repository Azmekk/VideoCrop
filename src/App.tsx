import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import VideoView from "./components/VideoView";
import { Button, Progress } from "antd";
import CutSegment from "./components/CutSegment";
import CropSegment from "./components/CropSegment";
import type { VideoCropPoints, VideoEditOptions, VideoInfo } from "./Logic/Interfaces";
import "./App.css";
import CompressSegment from "./components/CompressSegment";
import ResizeSegment from "./components/ResizeSegment";
import { initiateVideoCropPoints, videoPathIsValid } from "./Logic/Utils";
import { CropPointsContext } from "./Logic/GlobalContexts";
import VideoPathSelection from "./components/VideoPathSelection";

function App() {
  const [ffmpegExsts, setFfmpegExsts] = useState(true);
  const [interactingWithPaths, setInteractingWithPaths] = useState(false);
  const [videoPath, setVideoPath] = useState("");
  const [videoInfo, setVideoInfo] = useState<VideoInfo | undefined>(undefined);
  const [resetCropPoints, setResetCropPoints] = useState(0);

  const [processingVideo, setProcessingVideo] = useState(false);
  const [processingProgress, setProcessingProgress] = useState(0);

  const [cropPointPositions, setCropPointPositions] = useState<VideoCropPoints>(initiateVideoCropPoints());
  const [cropLinesEnabled, setCropLinesEnabled] = useState(false);

  const [videoEditOptions, setvideoEditOptions] = useState<VideoEditOptions>({
    output_video_path: "",
    cut_options_enabled: false,
    cut_options: { starting_time_string: "0:00:00.000", end_time_string: "0:00:00.000" },
    crop_enabled: false,
    crop_options: { starting_x_offset: 0, starting_y_offset: 0, width: 0, height: 0 },
    compression_enabled: false,
    compression_options: { codec: "libx264", preset: "medium", using_crf: true, crf: 23, bitrate: 5550 },
    resize_enabled: false,
    resize_options: { width: 0, height: 0 },
  });

  useEffect(() => {
    console.log(videoEditOptions);
  }, [videoEditOptions]);

  let video_selector_open = false;
  async function getVideoPath() {
    try {
      if (video_selector_open === true) {
        return;
      }

      setInteractingWithPaths(true);

      setvideoEditOptions({ ...videoEditOptions, output_video_path: "" });
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
        starting_x_offset: 0,
        starting_y_offset: 0,
        width: vidInfo.width,
        height: vidInfo.height,
      });

      console.log(vidInfo);
    } finally {
      setInteractingWithPaths(false);
    }
  }

  async function pickOutputPath() {
    try {
      setInteractingWithPaths(true);
      const path: string = await invoke("pick_output_path");
      setvideoEditOptions({ ...videoEditOptions, output_video_path: path });
    } finally {
      setInteractingWithPaths(false);
    }
  }

  async function submitVideo() {
    await invoke("submit_video_for_editing", { options: videoEditOptions });

    setProcessingVideo(true);
    setProcessingProgress(0);

    const interval = setInterval(() => {
      setProcessingProgress((prevProgress) => {
        if (prevProgress >= 100) {
          clearInterval(interval);
          setProcessingVideo(false);
          return 100;
        }
        return prevProgress + 10;
      });
    }, 1000);

    setProcessingProgress(0);
  }

  async function checkFfmpegAndFfprobe() {
    setFfmpegExsts(await invoke("check_ffmpeg_and_ffprobe"));
  }

  useEffect(() => {
    checkFfmpegAndFfprobe();
  }, []);

  if (!ffmpegExsts) {
    return <div className="app-disabled">FFmpeg and FFprobe were not located. Please download them and add them to path.</div>;
  }

  if (interactingWithPaths) {
    return <div className="app-disabled">Please select path or cancel selection before continuing.</div>;
  }

  return (
    <div>
      <main className={`app-container ${processingVideo && "disabled"} `}>
        <div className="general-video-options-container">
          <div style={{ width: "20%", display: "flex", flexDirection: "column", gap: "20px" }}>
            <div style={{ marginBottom: "20px" }}>
              <Button size="large" onClick={getVideoPath} type="primary">
                Select new video
              </Button>
            </div>
            <CompressSegment onChange={(x, enabled) => setvideoEditOptions({ ...videoEditOptions, compression_enabled: enabled, compression_options: x })} disabled={!videoPathIsValid(videoPath)} />
            <ResizeSegment
              onChange={(x, enabled) => setvideoEditOptions({ ...videoEditOptions, resize_enabled: enabled, resize_options: x })}
              disabled={!videoPathIsValid(videoPath)}
              videoInfo={videoInfo}
              videoNotCropped={videoEditOptions.crop_enabled === false}
            />
          </div>
          <CropPointsContext.Provider value={{ cropPointPositions, setCropPointPositions }}>
            <VideoView
              videoInfo={videoInfo}
              videoPath={videoPath}
              onVideoPathClick={(): void => {
                getVideoPath();
              }}
              resizerEnabled={cropLinesEnabled && videoEditOptions.crop_enabled}
              reset={resetCropPoints}
              cropEnabled={videoEditOptions.crop_enabled}
            />
          </CropPointsContext.Provider>
          <div style={{ width: "20%", display: "flex", flexDirection: "column", alignItems: "end" }}>
            <div>
              <VideoPathSelection videoEditOptions={videoEditOptions} videoPath={videoPath} onClick={pickOutputPath} />
              <CropPointsContext.Provider value={{ cropPointPositions, setCropPointPositions }}>
                <CropSegment
                  videoInfo={videoInfo}
                  onCropLinesEnabledChanged={(e) => setCropLinesEnabled(e)}
                  onSegmentEnabledChanged={(e) => setvideoEditOptions({ ...videoEditOptions, crop_enabled: e })}
                  disabled={!videoPathIsValid(videoPath)}
                  onReset={() => {
                    setResetCropPoints(resetCropPoints + 1);
                    setCropPointPositions({
                      starting_x_offset: 0,
                      starting_y_offset: 0,
                      width: videoInfo?.width ?? 0,
                      height: videoInfo?.height ?? 0,
                    });
                  }}
                  onChange={(x) => setvideoEditOptions({ ...videoEditOptions, crop_options: x })}
                />
              </CropPointsContext.Provider>
            </div>

            <div style={{ width: "100%", display: "flex", flexDirection: "column", alignItems: "end", justifyContent: "flex-end", height: "100%" }}>
              <Button loading={processingVideo} onClick={submitVideo} style={{ width: "80%" }} size="large" type="primary" disabled={videoEditOptions.output_video_path === ""}>
                Submit
              </Button>
            </div>
          </div>
        </div>

        <div className={videoPathIsValid(videoPath) ? "" : "disabled"}>
          <CutSegment
            onChange={(x, enabled) => setvideoEditOptions({ ...videoEditOptions, cut_options_enabled: enabled, cut_options: x })}
            videoPath={videoPath}
            videoDuration={videoInfo?.duration ?? "0:00:00.000"}
          />
        </div>
      </main>
      <div style={{ padding: "10px" }}>{processingVideo && <Progress percent={processingProgress} status="active" />}</div>
    </div>
  );
}

export default App;
