export function videoPathIsValid(videoPath: string){
    return (videoPath != "" && videoPath != "No file selected")
}

export interface VideoCropPoints{
  left: number,
  right: number,
  bottom: number,
  top: number
}