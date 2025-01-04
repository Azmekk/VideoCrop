import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import VideoView from "./components/VideoView";
import { Button, Progress } from "antd";
import CutSegment from "./components/CutSegment";
import CropSegment from "./components/CropSegment";
import type { DependenciesSetUpInfo, VideoCropPoints, VideoEditOptions, VideoEditProgress, VideoInfo } from "./Logic/Interfaces";
import "./App.css";
import CompressSegment from "./components/CompressSegment";
import ResizeSegment from "./components/ResizeSegment";
import { initiateVideoCropPoints, videoPathIsValid } from "./Logic/Utils";
import { CropPointsContext } from "./Logic/GlobalContexts";
import VideoPathSelection from "./components/VideoPathSelection";
import { platform } from "@tauri-apps/plugin-os";

function App() {
  const [ffmpegExists, setFfmpegExists] = useState(true);
  const [interactingWithPaths, setInteractingWithPaths] = useState(false);
  const [videoInfo, setVideoInfo] = useState<VideoInfo | undefined>(undefined);
  const [resetCropPoints, setResetCropPoints] = useState(0);
  const [depencenciesSetUpInfo, setDepencenciesSetUpInfo] = useState<DependenciesSetUpInfo>({ status: "", completed: false });

  const [processingVideo, setProcessingVideo] = useState(false);
  const [processingProgress, setProcessingProgress] = useState(0);

  const [cropPointPositions, setCropPointPositions] = useState<VideoCropPoints>(initiateVideoCropPoints());
  const [cropLinesEnabled, setCropLinesEnabled] = useState(false);

  const [currentOs, _] = useState(platform());
  const [downloadingDependencies, setDownloadingDependencies] = useState(false);

  const [videoEditOptions, setvideoEditOptions] = useState<VideoEditOptions>({
    input_video_path: "",
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

      setvideoEditOptions({ ...videoEditOptions, input_video_path: path });

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
      if (videoPathIsValid(path) === false) {
        return;
      }
      setvideoEditOptions({ ...videoEditOptions, output_video_path: path });
    } finally {
      setInteractingWithPaths(false);
    }
  }

  async function submitVideo() {
    await invoke("submit_video_for_editing", { options: videoEditOptions });

    setProcessingVideo(true);
    setProcessingProgress(0);

    while (true) {
      const newVideoInfo = await invoke<VideoEditProgress>("get_video_progress_info");

      if (newVideoInfo.last_error) {
        setProcessingVideo(false);
        setProcessingProgress(0);

        alert(`Something went wrong: ${newVideoInfo.last_error}`);
        break;
      }

      if (newVideoInfo.working) {
        setProcessingProgress(newVideoInfo.progress);
      }

      if (newVideoInfo.progress === 100) {
        setProcessingProgress(100);
        await new Promise((resolve) => setTimeout(resolve, 300));

        setProcessingVideo(false);
        setProcessingProgress(0);
      }

      await new Promise((resolve) => setTimeout(resolve, 50));
    }
  }

  async function checkFfmpegAndFfprobe() {
    setFfmpegExists(await invoke("check_ffmpeg_and_ffprobe"));
  }

  async function downloadDependencies() {
    try {
      setDownloadingDependencies(true);
      if (currentOs !== "windows") {
        return;
      }

      await new Promise((resolve) => setTimeout(resolve, 100));
      await invoke("download_ffmpeg_windows");

      while (true) {
        const depSetUpInfo = await invoke<DependenciesSetUpInfo>("get_depencencies_download_info");
        setDepencenciesSetUpInfo(depSetUpInfo);

        if (depSetUpInfo.completed) {
          break;
        }

        await new Promise((resolve) => setTimeout(resolve, 50));
      }
    } finally {
      setFfmpegExists(await invoke("check_ffmpeg_and_ffprobe"));
      setDownloadingDependencies(false);
    }
  }

  useEffect(() => {
    checkFfmpegAndFfprobe();
  }, []);

  return (
    <div>
      {!ffmpegExists && (
        <div className="app-disabled">
          FFmpeg and FFprobe were not located on path.
          {currentOs === "windows" && (
            <div>
              <Button loading={downloadingDependencies} onClick={downloadDependencies} size="large" type="primary">
                {downloadingDependencies ? <div>{depencenciesSetUpInfo.status}</div> : "Download for app only"}
              </Button>
            </div>
          )}
        </div>
      )}
      {interactingWithPaths && <div className="app-disabled">Please select path or cancel selection before continuing.</div>}
      <main className={`app-container ${processingVideo && "disabled"} `}>
        <div className="general-video-options-container">
          <div style={{ width: "20%", display: "flex", flexDirection: "column", gap: "20px" }}>
            <div style={{ marginBottom: "20px" }}>
              <Button size="large" onClick={getVideoPath} type="primary">
                Select new video
              </Button>
            </div>
            <CompressSegment
              onChange={(x, enabled) => setvideoEditOptions({ ...videoEditOptions, compression_enabled: enabled, compression_options: x })}
              disabled={!videoPathIsValid(videoEditOptions.input_video_path)}
            />
            <ResizeSegment
              onChange={(x, enabled) => setvideoEditOptions({ ...videoEditOptions, resize_enabled: enabled, resize_options: x })}
              disabled={!videoPathIsValid(videoEditOptions.input_video_path)}
              videoInfo={videoInfo}
              videoNotCropped={videoEditOptions.crop_enabled === false}
            />
          </div>
          <CropPointsContext.Provider value={{ cropPointPositions, setCropPointPositions }}>
            <VideoView
              videoInfo={videoInfo}
              videoPath={videoEditOptions.input_video_path}
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
              <VideoPathSelection videoEditOptions={videoEditOptions} videoPath={videoEditOptions.input_video_path} onClick={pickOutputPath} />
              <CropPointsContext.Provider value={{ cropPointPositions, setCropPointPositions }}>
                <CropSegment
                  videoInfo={videoInfo}
                  onCropLinesEnabledChanged={(e) => setCropLinesEnabled(e)}
                  onSegmentEnabledChanged={(e) => setvideoEditOptions({ ...videoEditOptions, crop_enabled: e })}
                  disabled={!videoPathIsValid(videoEditOptions.input_video_path)}
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

            <div style={{ width: "100%", display: "flex", gap: "10px", alignItems: "end", justifyContent: "center", height: "100%" }}>
              <Button loading={processingVideo} onClick={submitVideo} size="large" type="primary" disabled={videoEditOptions.output_video_path === ""}>
                Submit
              </Button>
              <Button type="primary" disabled={videoEditOptions.output_video_path === ""} size="large">
                Extract audio
              </Button>
            </div>
          </div>
        </div>

        <div className={videoPathIsValid(videoEditOptions.input_video_path) ? "" : "disabled"}>
          <CutSegment
            onChange={(x, enabled) => setvideoEditOptions({ ...videoEditOptions, cut_options_enabled: enabled, cut_options: x })}
            videoPath={videoEditOptions.input_video_path}
            videoDuration={videoInfo?.duration ?? "0:00:00.000"}
          />
        </div>
      </main>
      <div style={{ padding: "10px" }}>{processingVideo && <Progress percent={processingProgress} status="active" />}</div>
    </div>
  );
}

export default App;
