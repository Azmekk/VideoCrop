export interface VideoCropPoints {
  startingXOffset: number;
  startingYOffset: number;
  width: number;
  height: number;
}

export interface VideoCompressionOptions {
  codec: string;
  preset: string;
  usingCrf: boolean;
  crf: number | undefined;
  bitrate: number | undefined;
}

export interface VideoCutOptions {
  startingSecond: number;
  endingSecond: number;
}

export interface VideoInfo {
  width: number;
  height: number;
  duration: string;
}

export interface ResizeOptions {
  width: number;
  height: number;
}

export interface VideoEditOptions {
  cutOptionsEnabled: boolean;
  cutOptions: VideoCutOptions | undefined;
  cropLinesEnabled: boolean;
  cropPointsEnabled: boolean;
  cropPoints: VideoCropPoints | undefined;
  compressionEnabled: boolean;
  compressionOptions: VideoCompressionOptions | undefined;
  resizeEnabled: boolean;
  resizeOptions: ResizeOptions | undefined;
}

export interface VideoCropLineDisplacements {
  left: number;
  right: number;
  top: number;
  bottom: number;
}
