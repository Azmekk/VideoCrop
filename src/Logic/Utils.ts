import type { VideoCropPoints } from "./Interfaces";

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
