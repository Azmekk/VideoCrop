// Builds a MIME + codecs string from ffprobe metadata so `video.canPlayType` can confirm
// the backend's `directLikelyPlayable` hint (HEVC/AV1 support is per-machine).

import type { VideoInfo } from "../types";

function containerMime(fmt: string, path: string): string {
  const f = fmt.toLowerCase();
  const ext = path.toLowerCase().split(".").pop() ?? "";
  if (f.includes("webm") || ext === "webm") return "video/webm";
  if (f.includes("matroska") || ext === "mkv") return "video/x-matroska";
  if (ext === "mov") return "video/quicktime";
  return "video/mp4";
}

function h264Codec(profile: string | null): string {
  const p = (profile ?? "").toLowerCase();
  // avc1.PPCCLL — profile_idc, constraint flags, level (use a generous 4.0 level: canPlayType
  // answers on profile support, not exact level).
  if (p.includes("high 10")) return "avc1.6E0028";
  if (p.includes("high")) return "avc1.640028";
  if (p.includes("main")) return "avc1.4D4028";
  if (p.includes("baseline")) return "avc1.42E028";
  return "avc1.640028";
}

function videoCodec(info: VideoInfo): string | null {
  const c = info.videoCodec.toLowerCase();
  const tenBit = (info.pixFmt ?? "").includes("10");
  switch (c) {
    case "h264": return h264Codec(info.profile);
    case "hevc": case "h265":
      return tenBit ? "hvc1.2.4.L120.90" : "hvc1.1.6.L120.90";
    case "av1": return tenBit ? "av01.0.08M.10" : "av01.0.08M.08";
    case "vp9": return tenBit ? "vp09.02.40.10" : "vp09.00.40.08";
    case "vp8": return "vp8";
    default: return null;
  }
}

function audioCodec(codec: string): string | null {
  switch (codec.toLowerCase()) {
    case "aac": return "mp4a.40.2";
    case "mp3": return "mp4a.69";
    case "opus": return "opus";
    case "vorbis": return "vorbis";
    case "flac": return "flac";
    default: return null;
  }
}

/** e.g. `video/mp4; codecs="avc1.640028, mp4a.40.2"`, or null if the video codec is unknown. */
export function codecString(info: VideoInfo): string | null {
  const v = videoCodec(info);
  if (!v) return null;
  const a = info.audioStreams[0] ? audioCodec(info.audioStreams[0].codec) : null;
  if (info.audioStreams[0] && !a) return null;
  let mime = containerMime(info.containerFormat, info.path);
  // WebView2 reports "" for matroska/quicktime MIME even when the codecs decode; ask as mp4.
  if (mime !== "video/webm") mime = "video/mp4";
  return `${mime}; codecs="${a ? `${v}, ${a}` : v}"`;
}

/** True when the hint and the browser agree the file can be played directly. */
export function canPlayDirect(info: VideoInfo, hint: boolean, probe: HTMLVideoElement): boolean {
  // The backend says "no" to HEVC because support is per-machine (hardware + HEVC extension);
  // in an mp4/mov container let canPlayType decide instead.
  const hevcInMp4 = /^(hevc|h265)$/i.test(info.videoCodec) && /mp4|mov/i.test(info.containerFormat);
  if (!hint && !hevcInMp4) return false;
  const s = codecString(info);
  if (!s) return false;
  return probe.canPlayType(s) !== "";
}
