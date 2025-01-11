import { invoke } from "@tauri-apps/api/core";
import type { DependenciesSetUpInfo, VideoEditOptions, VideoEditProgress } from "../Interfaces/Interfaces";

export async function submitVideo(videoEditOptions: VideoEditOptions, setProcessingSubmission: (processingSubmission: boolean) => void, setProcessingProgress: (processingProgress: number) => void) {
  const videoEditOptionsLocal = videoEditOptions;

  await invoke("submit_video_for_editing", { options: videoEditOptionsLocal });

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

export async function submitAudioOnly(
  videoEditOptions: VideoEditOptions,
  setProcessingSubmission: (processingSubmission: boolean) => void,
  setProcessingProgress: (processingProgress: number) => void,
) {
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

export async function downloadDependencies(
  setDownloadingDependencies: (downloadingDependencies: boolean) => void,
  setDepencenciesSetUpInfo: (depencenciesSetUpInfo: DependenciesSetUpInfo) => void,
  setFfmpegExists: (ffmpegExists: boolean) => void,
  currentOs: string,
) {
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

      await new Promise((resolve) => setTimeout(resolve, 180));
    }
  } finally {
    setFfmpegExists(await invoke("check_ffmpeg_and_ffprobe"));
    setDownloadingDependencies(false);
  }
}
