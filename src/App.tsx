import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import VideoView from "./components/VideoView";
import { Button, Dropdown, type MenuProps, Modal, Progress } from "antd";
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
import { event } from "@tauri-apps/api";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { check } from "@tauri-apps/plugin-updater";
import { DownOutlined } from "@ant-design/icons";

function App() {
  const [ffmpegExists, setFfmpegExists] = useState(true);
  const [interactingWithPaths, setInteractingWithPaths] = useState(false);
  const [videoInfo, setVideoInfo] = useState<VideoInfo | undefined>(undefined);
  const [resetCropPoints, setResetCropPoints] = useState(0);
  const [depencenciesSetUpInfo, setDepencenciesSetUpInfo] = useState<DependenciesSetUpInfo>({ status: "", completed: false });
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [updatingApp, setUpdatingApp] = useState(false);
  const [draggedVideoPath, setDraggedVideoPath] = useState("");

  const [selectedSubmitDropdownType, setSelectedSubmitDropdownType] = useState("default");
  const [processingSubmission, setProcessingSubmission] = useState(false);
  const [processingProgress, setProcessingProgress] = useState(0);

  const [cropPointPositions, setCropPointPositions] = useState<VideoCropPoints>(initiateVideoCropPoints());
  const [cropLinesUnlocked, setCropLinesUnlocked] = useState(false);

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
    compression_options: { codec: "libx264", preset: "medium", using_crf: true, crf: 23, bitrate: 5550, audio_codec: "copy", audio_bitrate: 128 },
    resize_enabled: false,
    resize_options: { width: 0, height: 0 },
  });

  function isValidVideoFile(filePath: string): boolean {
    const validExtensions = ["mp4", "avi", "mov", "mkv", "webm"];
    const fileExtension = filePath.split(".").pop()?.toLowerCase();
    return validExtensions.includes(fileExtension ?? "");
  }

  useEffect(() => {
    console.log(videoEditOptions);
  }, [videoEditOptions]);

  useEffect(() => {
    setCropPointPositions({
      starting_x_offset: 0,
      starting_y_offset: 0,
      width: videoInfo?.width ?? 0,
      height: videoInfo?.height ?? 0,
    });
  }, [resetCropPoints]);

  const changeVideoPath = async (path: string) => {
    setvideoEditOptions({ ...videoEditOptions, input_video_path: path });

    const vidInfo: VideoInfo = await invoke("get_video_info", { videoPath: path });
    setVideoInfo(vidInfo);

    setCropPointPositions({
      starting_x_offset: 0,
      starting_y_offset: 0,
      width: vidInfo.width,
      height: vidInfo.height,
    });
  };

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

      await changeVideoPath(path);
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

  async function submitForProcessing() {
    if (selectedSubmitDropdownType === "audio") {
      await submitAudioOnly();
    } else {
      await submitVideo();
    }
  }

  async function submitVideo() {
    await invoke("submit_video_for_editing", { options: videoEditOptions, processAudio: selectedSubmitDropdownType !== "video" });

    setProcessingSubmission(true);
    setProcessingProgress(0);

    while (true) {
      await new Promise((resolve) => setTimeout(resolve, 300));
      const newVideoInfo = await invoke<VideoEditProgress>("get_video_progress_info");

      if (newVideoInfo.last_error) {
        setProcessingSubmission(false);
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

        setProcessingSubmission(false);
        setProcessingProgress(0);
      }

      if (!newVideoInfo.working) {
        setProcessingSubmission(false);
        setProcessingProgress(0);
        break;
      }

      await new Promise((resolve) => setTimeout(resolve, 50));
    }
  }

  async function submitAudioOnly() {
    await invoke("submit_audio_extraction", { options: videoEditOptions });

    setProcessingSubmission(true);
    setProcessingProgress(0);

    while (true) {
      await new Promise((resolve) => setTimeout(resolve, 300));

      const newVideoInfo = await invoke<VideoEditProgress>("get_video_progress_info");

      if (newVideoInfo.last_error) {
        setProcessingSubmission(false);
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

        setProcessingSubmission(false);
        setProcessingProgress(0);
      }

      if (!newVideoInfo.working) {
        setProcessingSubmission(false);
        setProcessingProgress(0);
        break;
      }

      await new Promise((resolve) => setTimeout(resolve, 50));
    }
  }

  async function checkFfmpegAndFfprobe() {
    setFfmpegExists(await invoke("check_ffmpeg_and_ffprobe"));
  }

  async function checkForUpdates() {
    const update_check = await check();
    console.log("Checking for updates", update_check !== null);
    setUpdateAvailable(update_check !== null);
  }

  async function updateApp() {
    try {
      setUpdatingApp(true);
      const update = await check();
      if (update) {
        console.log(`found update ${update.version} from ${update.date} with notes ${update.body}`);
        let downloaded = 0;
        let contentLength = 0;
        await update.downloadAndInstall((event) => {
          switch (event.event) {
            case "Started":
              contentLength = event.data.contentLength ?? 0;
              console.log(`started downloading ${event.data.contentLength} bytes`);
              break;
            case "Progress":
              downloaded += event.data.chunkLength;
              console.log(`downloaded ${downloaded} from ${contentLength}`);
              break;
            case "Finished":
              console.log("download finished");
              break;
          }
        });

        console.log("update installed");
      }
    } catch (error) {
      alert(`Error while updating: ${error}`);
    } finally {
      setUpdatingApp(false);
      setUpdateAvailable(false);
    }
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
    checkForUpdates();
    checkFfmpegAndFfprobe();

    let unlisten_drag_drop: UnlistenFn | undefined = undefined;
    let unlisten_drag_in: UnlistenFn | undefined = undefined;
    let unlisten_drag_out: UnlistenFn | undefined = undefined;

    const startListeningForDrop = async () => {
      unlisten_drag_drop = await event.listen(event.TauriEvent.DRAG_DROP, (e) => {
        const payload = e.payload as { paths: string[] };
        if (isValidVideoFile(payload.paths[0]) === true) {
          changeVideoPath(payload.paths[0]);
        }
        setDraggedVideoPath("");
        console.log(e);
      });

      unlisten_drag_in = await event.listen(event.TauriEvent.DRAG_ENTER, (e) => {
        const payload = e.payload as { paths: string[] };
        if (isValidVideoFile(payload.paths[0]) === false) {
          setDraggedVideoPath("invalid");
          return;
        }
        setDraggedVideoPath(payload.paths[0]);
        console.log(e);
      });

      unlisten_drag_out = await event.listen(event.TauriEvent.DRAG_LEAVE, (e) => {
        setDraggedVideoPath("");
        console.log(e);
      });
    };

    startListeningForDrop();

    return () => {
      if (unlisten_drag_drop) {
        unlisten_drag_drop();
      }

      if (unlisten_drag_in) {
        unlisten_drag_in();
      }

      if (unlisten_drag_out) {
        unlisten_drag_out();
      }
    };
  }, []);

  const codecDropdownItems: { key: string; label: string }[] = [
    {
      key: "default",
      label: "Submit",
    },
    {
      key: "audio",
      label: "Audio Only",
    },
    {
      key: "video",
      label: "Video Only",
    },
  ];

  const handleProcessingDropdownMenuSelection: MenuProps["onClick"] = (e) => {
    setSelectedSubmitDropdownType(e.key);
  };

  const selectedSubmitDropdownTypeProps: MenuProps = {
    items: codecDropdownItems,
    onClick: handleProcessingDropdownMenuSelection,
    selectable: true,
    selectedKeys: [selectedSubmitDropdownType],
    defaultSelectedKeys: ["libx264"],
  };

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
      {draggedVideoPath !== "" && <div className="dragged-file-blur">{draggedVideoPath === "invalid" ? <div className="dragged-file-invalid">Invalid file</div> : draggedVideoPath}</div>}
      {interactingWithPaths && <div className="app-disabled">Please select path or cancel selection before continuing.</div>}
      <main className={`app-container ${processingSubmission && "disabled"} `}>
        <div className="general-video-options-container">
          <div style={{ width: "20%", display: "flex", flexDirection: "column", gap: "20px" }}>
            <div style={{ display: "flex", flexDirection: "column", gap: "7px", marginBottom: "20px", maxWidth: "80%" }}>
              <Button onClick={getVideoPath} type="primary">
                {videoPathIsValid(videoEditOptions.input_video_path) ? "Change video" : "Select video"}
              </Button>
              <Dropdown.Button onClick={submitForProcessing} icon={<DownOutlined />} disabled={videoEditOptions.output_video_path === ""} menu={selectedSubmitDropdownTypeProps}>
                {codecDropdownItems.find((item) => item.key === selectedSubmitDropdownType)?.label}
              </Dropdown.Button>
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
          <CropPointsContext.Provider
            value={{ cropPointPositions, setCropPointPositions, cropLinesUnlocked, setCropLinesUnlocked, cropEnabled: videoEditOptions.crop_enabled, resetCropPoints, setResetCropPoints }}
          >
            <VideoView
              videoInfo={videoInfo}
              videoPath={videoEditOptions.input_video_path}
              onVideoPathClick={(): void => {
                getVideoPath();
              }}
            />
            <div style={{ width: "20%", display: "flex", flexDirection: "column", alignItems: "end" }}>
              <div>
                <VideoPathSelection videoEditOptions={videoEditOptions} videoPath={videoEditOptions.input_video_path} onClick={pickOutputPath} />
                <CropSegment
                  videoInfo={videoInfo}
                  onCropLinesLockStateChanged={(e) => setCropLinesUnlocked(e)}
                  onSegmentEnabledChanged={(e) => setvideoEditOptions({ ...videoEditOptions, crop_enabled: e })}
                  disabled={!videoPathIsValid(videoEditOptions.input_video_path)}
                  onReset={() => {
                    setResetCropPoints(resetCropPoints + 1);
                  }}
                  onChange={(x) => setvideoEditOptions({ ...videoEditOptions, crop_options: x })}
                />
              </div>
            </div>
          </CropPointsContext.Provider>
        </div>

        <div className={videoPathIsValid(videoEditOptions.input_video_path) ? "" : "disabled"}>
          <CutSegment
            onChange={(x, enabled) => setvideoEditOptions({ ...videoEditOptions, cut_options_enabled: enabled, cut_options: x })}
            videoPath={videoEditOptions.input_video_path}
            videoDuration={videoInfo?.duration ?? "0:00:00.000"}
          />
        </div>
      </main>
      <div style={{ padding: "5px" }}>{processingSubmission && <Progress percent={processingProgress} status="active" />}</div>
      <Modal
        title="Update available. Would you like to update?"
        open={updateAvailable}
        footer={[
          <Button loading={updatingApp} onClick={async () => await updateApp()} key="yes" type="primary">
            Yes
          </Button>,
          <Button loading={updatingApp} onClick={() => setUpdateAvailable(false)} key="no" type="default">
            No
          </Button>,
        ]}
      />
    </div>
  );
}

export default App;
