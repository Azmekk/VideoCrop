import type { VideoCropPoints } from "./Interfaces"

export function videoPathIsValid(videoPath: string){
    return (videoPath !== "" && videoPath !== "No file selected")
}

export function initiateVideoCropPoints() : VideoCropPoints{
  return {
    topLeft: { x: 0, y: 0 },
    topRight: { x: 0, y: 0 },
    bottomLeft: { x: 0, y: 0 },
    bottomRight: { x: 0, y: 0 }
  }
}