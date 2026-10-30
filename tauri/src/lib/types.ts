// Mirror of src-tauri/src/core/models.rs. Keep the two in sync.

export interface VideoInfo {
  path: string;
  durationSec: number;
  width: number;
  height: number;
  sarNum: number;
  sarDen: number;
  videoCodec: string;
  profile: string | null;
  pixFmt: string | null;
  fps: number;
  bitrateBps: number | null;
  audioStreams: AudioStreamInfo[];
  containerFormat: string;
  fileSizeBytes: number | null;
}

export interface AudioStreamInfo {
  index: number;
  codec: string;
  channels: number | null;
  sampleRate: number | null;
  bitrateBps: number | null;
  language: string | null;
}

export interface CutSpec {
  startSec: number;
  endSec: number;
  accurate: boolean;
}

/** Source-pixel coordinates. */
export interface CropSpec {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** Applies to the post-crop frame. */
export interface ResizeSpec {
  width: number;
  height: number;
  aspectLocked: boolean;
}

export type VideoCodec = "h264" | "h265" | "av1" | "vp9";
export type AudioCodec = "aac" | "opus" | "mp3" | "copy";
export type RateControl = "crf" | "cbr" | "vbr" | "targetSize";
export type SpeedPreset =
  | "ultrafast" | "superfast" | "veryfast" | "faster" | "fast"
  | "medium" | "slow" | "slower" | "veryslow";
export type PixelFormat = "yuv420p" | "yuv420p10le";

export interface CompressionSpec {
  codec: VideoCodec;
  rateControl: RateControl;
  crf: number;
  targetBitrateBps: number | null;
  targetSizeBytes: number | null;
  speed: SpeedPreset;
  audioCodec: AudioCodec;
  audioBitrateKbps: number;
  pixelFormat: PixelFormat;
  container: string | null;
}

export type CompressionChoice =
  | { kind: "off" }
  | { kind: "quality"; presetId: string }
  | { kind: "platform"; platformId: string }
  | { kind: "custom"; spec: CompressionSpec };

export type OutputMode = "everything" | "audioOnly" | "videoOnly";

export interface QualityPreset {
  id: string;
  name: string;
  category: "compatibility" | "optimization";
  description: string;
  warning: string | null;
  spec: CompressionSpec;
}

export interface PlatformPreset {
  id: string;
  name: string;
  icon: string;
  description: string;
  maxSizeBytes: number | null;
  maxWidth: number | null;
  maxHeight: number | null;
  spec: CompressionSpec;
}

export interface PresetCatalog {
  quality: QualityPreset[];
  platforms: PlatformPreset[];
  defaultQualityId: string;
}

export interface JobDraft {
  cut: CutSpec | null;
  crop: CropSpec | null;
  resize: ResizeSpec | null;
  compression: CompressionChoice;
  mode: OutputMode;
  outputDir: string | null;
}

export interface JobPlan {
  outputPath: string;
  summary: string;
  outputWidth: number | null;
  outputHeight: number | null;
  spec: CompressionSpec | null;
  crop: CropSpec | null;
  resize: ResizeSpec | null;
  cut: CutSpec | null;
  mode: OutputMode;
  twoPass: boolean;
  estimatedSizeBytes: number | null;
  warnings: string[];
  /** Size target can't be met at watchable quality; confirm before encoding. */
  sizeAlert: string | null;
}

export interface EncodeStatus {
  fraction: number;
  pass: number;
  passCount: number;
  fps: number;
  speed: number;
  etaSec: number | null;
}

export interface EncodeResult {
  ok: boolean;
  cancelled: boolean;
  outputPath: string;
  outputSizeBytes: number | null;
  error: string | null;
  stderrTail: string | null;
}

export interface ToolStatus {
  ffmpeg: string | null;
  ffprobe: string | null;
  ready: boolean;
  toolsDir: string;
  /** "bundled" (downloaded by VideoCrop), "path" or "dev". */
  source: string | null;
  /** Why the ffmpeg found on PATH can't be used, if one was found. */
  problem: string | null;
}

export interface ToolDownloadProgress {
  fraction: number;
  status: string;
}

export interface OpenedVideo {
  info: VideoInfo;
  session: string;
  directUrl: string;
  hlsUrl: string;
  /** `${frameUrl}?t=<sec>&w=<px>&fast=<0|1>` returns image/jpeg. */
  frameUrl: string;
  directLikelyPlayable: boolean;
}

export interface Settings {
  theme: "system" | "light" | "dark";
  lastOutputDir: string | null;
  defaultCompression: CompressionChoice | null;
  recentFiles: string[];
  previewEncoder: string | null;
  /** Release version the user chose "Skip this version" for. */
  skippedUpdate: string | null;
}

/** A newer GitHub release (notification only; the user downloads it themselves). */
export interface UpdateInfo {
  version: string;
  url: string;
  notes: string | null;
}
