// DEV-ONLY mock backend for iterating on the UI in a plain browser (`pnpm dev`).
// Loaded via dynamic import behind `import.meta.env.DEV`, so it never ships.
// URL flags: `?stream` forces the HLS path, `?tools=0` shows the first-run download screen,
// `?fail` makes encodes fail.

import type {
  CompressionSpec, EncodeResult, EncodeStatus, JobDraft, JobPlan, OpenedVideo, PresetCatalog,
  QualityPreset, PlatformPreset, Settings, ToolStatus,
} from "../types";

const flags = new URLSearchParams(location.search);
const listeners = new Map<string, Set<(p: unknown) => void>>();
const emit = (ev: string, payload: unknown) => listeners.get(ev)?.forEach((cb) => cb(payload));

export function listen<T>(event: string, cb: (p: T) => void) {
  let set = listeners.get(event);
  if (!set) listeners.set(event, (set = new Set()));
  const f = cb as (p: unknown) => void;
  set.add(f);
  return Promise.resolve(() => { set!.delete(f); });
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

const spec = (o: Partial<CompressionSpec>): CompressionSpec => ({
  codec: "h264", rateControl: "crf", crf: 23, targetBitrateBps: null, targetSizeBytes: null,
  speed: "medium", audioCodec: "aac", audioBitrateKbps: 128, pixelFormat: "yuv420p", container: null, ...o,
});

const quality: QualityPreset[] = [
  { id: "h264-balanced", name: "H.264 balanced", category: "compatibility", description: "Plays everywhere. Good quality at a moderate size.", warning: null, spec: spec({}) },
  { id: "h264-high", name: "H.264 high quality", category: "compatibility", description: "Visually lossless for most content. Larger files.", warning: null, spec: spec({ crf: 18, speed: "slow", audioBitrateKbps: 192 }) },
  { id: "h264-small", name: "H.264 small", category: "compatibility", description: "Noticeably softer, but tiny. Fine for quick shares.", warning: null, spec: spec({ crf: 28, speed: "fast", audioBitrateKbps: 96 }) },
  { id: "h265-balanced", name: "H.265 efficient", category: "optimization", description: "About 40% smaller than H.264 at the same quality.", warning: "Some older browsers, TVs and Windows installs without the HEVC extension can't play H.265.", spec: spec({ codec: "h265", crf: 26 }) },
  { id: "av1-small", name: "AV1 smallest", category: "optimization", description: "Best compression available. Slow to encode.", warning: "AV1 needs recent hardware or software to play smoothly; older phones may stutter.", spec: spec({ codec: "av1", crf: 32, speed: "medium", audioCodec: "opus" }) },
  { id: "vp9-web", name: "VP9 for the web", category: "optimization", description: "Royalty-free WebM. Good for embedding on websites.", warning: "WebM doesn't play in QuickTime or on older iPhones.", spec: spec({ codec: "vp9", crf: 33, audioCodec: "opus", container: "webm" }) },
];

const MB = 1024 * 1024;
const plat = (id: string, name: string, icon: string, description: string, cap: number | null, w: number | null, h: number | null, s: Partial<CompressionSpec>): PlatformPreset =>
  ({ id, name, icon, description, maxSizeBytes: cap ? cap * MB : null, maxWidth: w, maxHeight: h, spec: spec(s) });

const platforms: PlatformPreset[] = [
  plat("discord", "Discord", "discord", "Fits the free upload limit.", 10, 1920, 1080, {}),
  plat("discord-nitro-basic", "Discord Nitro Basic", "discord", "Nitro Basic upload limit.", 50, 1920, 1080, {}),
  plat("discord-nitro", "Discord Nitro", "discord", "Nitro upload limit, up to 1440p.", 500, 2560, 1440, { audioBitrateKbps: 160 }),
  plat("youtube", "YouTube", "youtube", "High-quality master for upload.", null, null, null, { crf: 18, speed: "slow", audioBitrateKbps: 320 }),
  plat("x", "X / Twitter", "x", "Within X's size and resolution limits.", 512, 1920, 1200, {}),
  plat("vertical", "TikTok / Reels / Shorts", "vertical", "Vertical 1080×1920.", null, 1080, 1920, { crf: 21 }),
  plat("whatsapp", "WhatsApp", "whatsapp", "Small enough to send in chat.", 16, 1280, 720, { audioBitrateKbps: 96 }),
  plat("email", "Email", "email", "Fits most attachment limits.", 25, 1280, 720, { audioBitrateKbps: 96 }),
];

const catalog: PresetCatalog = { quality, platforms, defaultQualityId: "h264-balanced" };

let settings: Settings = (() => {
  try { const s = localStorage.getItem("vc-mock-settings"); if (s) return JSON.parse(s); } catch {}
  return { theme: "system", lastOutputDir: null, defaultCompression: null, recentFiles: ["C:\\Videos\\OBS\\2026-10-28 21-04-11.mkv", "C:\\Videos\\Clips\\boss fight.mp4"], previewEncoder: null, skippedUpdate: null };
})();

let toolsReady = flags.get("tools") !== "0";
let opened: OpenedVideo | null = null;
let cancelFlag = false;

function plan(d: JobDraft): JobPlan {
  if (!opened) throw "No video is open.";
  const info = opened.info;
  const cut = d.cut ?? { startSec: 0, endSec: info.durationSec, accurate: false };
  const dur = cut.endSec - cut.startSec;
  const c = d.compression;
  const lines: string[] = [];
  const warnings: string[] = [];
  let s: CompressionSpec | null = null;
  let twoPass = false;
  let w = d.crop?.width ?? info.width, h = d.crop?.height ?? info.height;
  if (c.kind === "quality") s = quality.find((q) => q.id === c.presetId)?.spec ?? quality[0].spec;
  if (c.kind === "custom") s = c.spec;
  if (c.kind === "platform") {
    const p = platforms.find((x) => x.id === c.platformId)!;
    s = p.spec;
    twoPass = p.maxSizeBytes != null;
    if (p.maxHeight && h > p.maxHeight) { w = Math.round((w * p.maxHeight) / h / 2) * 2; h = p.maxHeight; }
    if (p.id === "vertical" && w >= h) warnings.push("Tip: crop to 9:16 for a full-screen vertical video.");
    if (p.maxSizeBytes && dur > 120) warnings.push(`Clip is long for ${p.maxSizeBytes / MB} MB — consider cutting it.`);
  }
  if (d.cut) lines.push(`Cut ${cut.startSec.toFixed(3)}s → ${cut.endSec.toFixed(3)}s (${dur.toFixed(2)} s, ${cut.accurate ? "accurate" : "fast, keyframe-snapped"})`);
  const encode = c.kind !== "off" && d.mode !== "audioOnly";
  if (d.crop && encode) lines.push(`Crop ${d.crop.width}×${d.crop.height} at ${d.crop.x},${d.crop.y}`);
  if (d.resize && encode) { w = d.resize.width; h = d.resize.height; lines.push(`Scale to ${w}×${h} (lanczos)`); }
  if (!encode && c.kind === "off") lines.push(d.crop || d.resize ? "Stream copy — crop and resize are skipped" : "Stream copy (no re-encode)");
  else if (s && d.mode !== "audioOnly") lines.push(`${s.codec.toUpperCase()} ${twoPass ? "two-pass target size" : `CRF ${s.crf}`}, ${s.speed}`);
  if (d.mode !== "videoOnly") lines.push(s ? `Audio ${s.audioCodec.toUpperCase()} ${s.audioBitrateKbps} kbps` : "Audio copied");
  else lines.push("No audio");
  const ext = d.mode === "audioOnly" ? "m4a" : s?.container ?? (s?.codec === "vp9" ? "webm" : "mp4");
  const dir = d.outputDir ?? "C:\\Videos";
  const kbps = s ? (s.crf < 20 ? 9000 : s.crf < 25 ? 5000 : 2500) * ((w * h) / (1920 * 1080)) : (info.bitrateBps ?? 4e6) / 1000;
  let est = ((kbps + (s?.audioBitrateKbps ?? 128)) * 1000 / 8) * dur;
  if (c.kind === "platform") { const cap = platforms.find((x) => x.id === c.platformId)!.maxSizeBytes; if (cap) est = Math.min(est, cap * 0.97); }
  if (d.mode === "audioOnly") est = (s?.audioBitrateKbps ?? 160) * 125 * dur;
  return {
    outputPath: `${dir}\\Sample recording_VideoCrop.${ext}`,
    summary: lines.join("\n"),
    outputWidth: d.mode === "audioOnly" ? null : w, outputHeight: d.mode === "audioOnly" ? null : h,
    spec: s, crop: d.crop, resize: d.resize, cut: d.cut, mode: d.mode, twoPass,
    estimatedSizeBytes: Math.round(est), warnings,
    // `?sizealert` previews the confirm-before-encoding dialog on any capped share preset.
    sizeAlert: c.kind === "platform" && new URLSearchParams(location.search).has("sizealert")
      ? "A 2 h clip can't fit in 10 MB: the audio alone needs about 115 MB. It'll come out around 135 MB instead, and look rough. Cut it shorter or pick a bigger size limit."
      : null,
  };
}

async function runEncode(p: JobPlan) {
  cancelFlag = false;
  const passes = p.twoPass ? 2 : 1;
  for (let pass = 1; pass <= passes; pass++) {
    for (let i = 0; i <= 100; i++) {
      if (cancelFlag) {
        emit("encode://done", { ok: false, cancelled: true, outputPath: p.outputPath, outputSizeBytes: null, error: null, stderrTail: null } satisfies EncodeResult);
        return;
      }
      if (flags.has("fail") && i === 40) {
        emit("encode://done", { ok: false, cancelled: false, outputPath: p.outputPath, outputSizeBytes: null, error: "ffmpeg exited with code 1: Invalid argument", stderrTail: "[libx264 @ 0000020] height not divisible by 2 (1600x901)\nError initializing output stream 0:0 -- Error while opening encoder\nConversion failed!" } satisfies EncodeResult);
        return;
      }
      const s: EncodeStatus = { fraction: i / 100, pass, passCount: passes, fps: 140 + Math.random() * 20, speed: 4.6 + Math.random() * 0.4, etaSec: ((100 - i) + (passes - pass) * 100) * 0.05 };
      emit("encode://progress", s);
      await sleep(50);
    }
  }
  emit("encode://done", { ok: true, cancelled: false, outputPath: p.outputPath, outputSizeBytes: p.estimatedSizeBytes, error: null, stderrTail: null } satisfies EncodeResult);
}

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  await sleep(cmd === "plan_job" ? 5 : 60);
  const r = ((): unknown => {
    switch (cmd) {
      case "tools_status": return { ffmpeg: toolsReady ? "C:\\tools\\ffmpeg.exe" : null, ffprobe: toolsReady ? "C:\\tools\\ffprobe.exe" : null, ready: toolsReady, toolsDir: "C:\\Users\\you\\AppData\\Local\\VideoCrop\\tools", source: toolsReady ? "path" : null, problem: toolsReady ? null : "The ffmpeg on your PATH (C:/ffmpeg/bin/ffmpeg.exe) is missing libsvtav1, libx265 (VideoCrop needs a full GPL build)." } satisfies ToolStatus;
      case "tools_download": return (async () => {
        const failOnce = flags.get("tools") === "0" && !sessionStorage.getItem("vc-mock-tools-failed");
        for (let i = 0; i <= 100; i += 2) {
          if (failOnce && i === 46) { sessionStorage.setItem("vc-mock-tools-failed", "1"); throw "Download failed: the connection was reset. Check your internet connection and try again."; }
          emit("tools://progress", { fraction: i / 100, status: i < 90 ? `Downloading ffmpeg… ${Math.round(i * 1.4)} of 140 MB` : "Extracting…" });
          await sleep(40);
        }
        toolsReady = true;
        return { ffmpeg: "C:\\tools\\ffmpeg.exe", ffprobe: "C:\\tools\\ffprobe.exe", ready: true, toolsDir: "C:\\tools", source: "bundled", problem: null } satisfies ToolStatus;
      })();
      case "open_video": {
        const path = String(args?.path ?? "sample.mp4");
        opened = {
          info: {
            path, durationSec: 30, width: 1920, height: 1080, sarNum: 1, sarDen: 1, videoCodec: "h264", profile: "High", pixFmt: "yuv420p", fps: 30,
            bitrateBps: 3_000_000, containerFormat: "mov,mp4,m4a,3gp,3g2,mj2", fileSizeBytes: 11_397_281,
            audioStreams: [{ index: 1, codec: "aac", channels: 1, sampleRate: 44100, bitrateBps: 96000, language: null }],
          },
          session: "mock",
          directUrl: "/dev/sample.mp4",
          hlsUrl: "/dev/hls/index.m3u8",
          frameUrl: "/dev/frame",
          directLikelyPlayable: !flags.has("stream"),
        };
        settings.recentFiles = [path, ...settings.recentFiles.filter((p) => p !== path)].slice(0, 8);
        return opened;
      }
      case "close_video": opened = null; return undefined;
      case "presets": return catalog;
      case "plan_job": return plan(args!.draft as JobDraft);
      case "encode_start": { const p = plan(args!.draft as JobDraft); void runEncode(p); return p; }
      case "encode_cancel": cancelFlag = true; return undefined;
      case "settings_get": return structuredClone(settings);
      case "settings_set": settings = args!.settings as Settings; try { localStorage.setItem("vc-mock-settings", JSON.stringify(settings)); } catch {} return undefined;
      case "reveal_in_folder": console.info("[mock] reveal", args?.path); return undefined;
      case "check_update":
        return new URLSearchParams(location.search).has("update")
          ? { version: "2.1.0", url: "https://videocrop.vexbyte.com/", notes: "- Faster previews\n- New share presets" }
          : null;
      case "open_release_page": console.info("[mock] open", args?.url); return undefined;
      default: throw `Unknown command ${cmd}`;
    }
  })();
  return (await r) as T;
}
