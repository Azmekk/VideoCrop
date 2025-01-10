import type { VideoCropPoints } from "../Interfaces/Interfaces";

export function videoPathIsValid(videoPath: string) {
  return videoPath !== "" && videoPath !== "No file selected" && videoPath !== "No path selected";
}

export function initiateVideoCropPoints(): VideoCropPoints {
  return {
    starting_x_offset: 0,
    starting_y_offset: 0,
    width: 0,
    height: 0,
  };
}

export const getCanvasToVideoSizeDifference = (videoRef: React.RefObject<HTMLVideoElement>) => {
  if (!videoRef.current) return { widthDiff: 0, heightDiff: 0 };

  const videoWidth = videoRef.current.videoWidth;
  const videoHeight = videoRef.current.videoHeight;

  const rect = videoRef.current.getBoundingClientRect();
  const elementWidth = rect.width;
  const elementHeight = rect.height;

  return {
    widthDiff: elementWidth / videoWidth,
    heightDiff: elementHeight / videoHeight,
  };
};
