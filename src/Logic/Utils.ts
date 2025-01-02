import type { VideoCropPoints } from "./Interfaces";

export function videoPathIsValid(videoPath: string) {
  return videoPath !== "" && videoPath !== "No file selected";
}

export function initiateVideoCropPoints(): VideoCropPoints {
  return {
    startingXOffset: 0,
    startingYOffset: 0,
    width: 0,
    height: 0,
  };
}
