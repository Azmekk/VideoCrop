// The only place the frontend talks to Rust. Command names match `#[tauri::command]` fns in
// src-tauri/src/commands.rs; event names match the `emit` calls in src-tauri/src/media/.
//
// In plain-browser dev (`pnpm dev` outside Tauri) calls are routed to a mock backend. The mock is
// behind `import.meta.env.DEV` + a dynamic import, so it is tree-shaken from production builds.

import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  EncodeResult, EncodeStatus, JobDraft, JobPlan, OpenedVideo, PresetCatalog,
  Settings, ToolDownloadProgress, ToolStatus, UpdateInfo,
} from "./types";

export const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
const useMock = import.meta.env.DEV && !inTauri;

type MockModule = typeof import("./mock/backend");
let mockP: Promise<MockModule> | null = null;
const mock = () => (mockP ??= import("./mock/backend"));

function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (useMock) return mock().then((m) => m.invoke<T>(cmd, args));
  return tauriInvoke<T>(cmd, args);
}

function listen<T>(event: string, cb: (payload: T) => void): Promise<UnlistenFn> {
  if (useMock) return mock().then((m) => m.listen<T>(event, cb));
  return tauriListen<T>(event, (e) => cb(e.payload));
}

export const api = {
  toolsStatus: () => invoke<ToolStatus>("tools_status"),
  /** Downloads ffmpeg + ffprobe; progress arrives on `onToolsProgress`. */
  toolsDownload: () => invoke<ToolStatus>("tools_download"),

  /** Probes the file and starts a media session; closes any previous session. */
  openVideo: (path: string) => invoke<OpenedVideo>("open_video", { path }),
  closeVideo: () => invoke<void>("close_video"),

  presets: () => invoke<PresetCatalog>("presets"),
  /** Pure + cheap: call on every edit (debounced) to refresh the summary/output name. */
  planJob: (draft: JobDraft) => invoke<JobPlan>("plan_job", { draft }),

  /** Re-plans server-side and runs. Progress/done arrive as events. */
  encodeStart: (draft: JobDraft) => invoke<JobPlan>("encode_start", { draft }),
  encodeCancel: () => invoke<void>("encode_cancel"),

  settingsGet: () => invoke<Settings>("settings_get"),
  settingsSet: (settings: Settings) => invoke<void>("settings_set", { settings }),

  /** Opens Explorer with the file selected. */
  revealInFolder: (path: string) => invoke<void>("reveal_in_folder", { path }),

  /** Newer GitHub release, or null when up to date / offline. */
  checkUpdate: () => invoke<UpdateInfo | null>("check_update"),
  /** Opens a VideoCrop release page in the default browser. */
  openReleasePage: (url: string) => invoke<void>("open_release_page", { url }),
};

export const events = {
  onEncodeProgress: (cb: (s: EncodeStatus) => void): Promise<UnlistenFn> =>
    listen<EncodeStatus>("encode://progress", cb),
  onEncodeDone: (cb: (r: EncodeResult) => void): Promise<UnlistenFn> =>
    listen<EncodeResult>("encode://done", cb),
  onToolsProgress: (cb: (p: ToolDownloadProgress) => void): Promise<UnlistenFn> =>
    listen<ToolDownloadProgress>("tools://progress", cb),
};

/** Builds a still-frame URL (`image/jpeg`) from `OpenedVideo.frameUrl`. `w` omitted = native size. */
export function frameSrc(frameUrl: string, t: number, w?: number, fast = true): string {
  const q = `t=${t.toFixed(3)}${w ? `&w=${w}` : ""}&fast=${fast ? 1 : 0}`;
  return `${frameUrl}${frameUrl.includes("?") ? "&" : "?"}${q}`;
}
