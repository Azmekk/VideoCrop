export interface VideoCropPoints{
  topLeft: CropPointPosition,
  topRight: CropPointPosition,
  bottomLeft: CropPointPosition,
  bottomRight: CropPointPosition
}

export interface CropPointPosition{
  x: number,
  y: number
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
  width: number,
  height: number,
  duration: string,
}

export interface ResizeOptions {
  width: number,
  height: number,
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