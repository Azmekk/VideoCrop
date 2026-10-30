// The only place the frontend talks to Rust. Command names match `#[tauri::command]` fns in
// src-tauri/src/commands.rs; event names match the `emit` calls in src-tauri/src/media/.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  EncodeResult, EncodeStatus, JobDraft, JobPlan, OpenedVideo, PresetCatalog,
  Settings, ToolDownloadProgress, ToolStatus,
} from "./types";

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
};

export const events = {
  onEncodeProgress: (cb: (s: EncodeStatus) => void): Promise<UnlistenFn> =>
    listen<EncodeStatus>("encode://progress", (e) => cb(e.payload)),
  onEncodeDone: (cb: (r: EncodeResult) => void): Promise<UnlistenFn> =>
    listen<EncodeResult>("encode://done", (e) => cb(e.payload)),
  onToolsProgress: (cb: (p: ToolDownloadProgress) => void): Promise<UnlistenFn> =>
    listen<ToolDownloadProgress>("tools://progress", (e) => cb(e.payload)),
};
