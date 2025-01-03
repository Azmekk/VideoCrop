export interface VideoCropPoints {
  starting_x_offset: number;
  starting_y_offset: number;
  width: number;
  height: number;
}

export interface VideoCompressionOptions {
  codec: string;
  preset: string;
  using_crf: boolean;
  crf: number;
  bitrate: number;
}

export interface VideoCutOptions {
  starting_time_string: string;
  end_time_string: string;
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
  output_video_path: string;
  cut_options_enabled: boolean;
  cut_options: VideoCutOptions;
  crop_enabled: boolean;
  crop_options: VideoCropPoints;
  compression_enabled: boolean;
  compression_options: VideoCompressionOptions;
  resize_enabled: boolean;
  resize_options: ResizeOptions;
}

export interface VideoCropLineDisplacements {
  left: number;
  right: number;
  top: number;
  bottom: number;
}
