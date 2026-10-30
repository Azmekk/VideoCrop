// Window, dialog and drag-drop helpers. Each is a no-op (or browser fallback) outside Tauri,
// and the Tauri modules are imported lazily so they stay off the startup path.

import { inTauri } from "./api";

export const VIDEO_EXTENSIONS = [
  "mp4", "m4v", "mov", "mkv", "webm", "avi", "wmv", "flv", "ts", "mts", "m2ts", "mpg", "mpeg", "3gp", "ogv",
];

const win = () => import("@tauri-apps/api/window").then((m) => m.getCurrentWindow());

export const windowControls = {
  minimize: () => inTauri && win().then((w) => w.minimize()),
  toggleMaximize: () => inTauri && win().then((w) => w.toggleMaximize()),
  close: () => (inTauri ? win().then((w) => w.close()) : undefined),
  show: () => inTauri && win().then((w) => w.show()),
  async onResized(cb: (maximized: boolean) => void): Promise<() => void> {
    if (!inTauri) return () => {};
    const w = await win();
    cb(await w.isMaximized());
    return w.onResized(async () => cb(await w.isMaximized()));
  },
};

export async function pickVideo(): Promise<string | null> {
  if (!inTauri) return import.meta.env.DEV ? "C:\\Videos\\Sample recording.mp4" : null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const r = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Videos", extensions: VIDEO_EXTENSIONS }, { name: "All files", extensions: ["*"] }],
  });
  return typeof r === "string" ? r : null;
}

export async function pickFolder(defaultPath?: string | null): Promise<string | null> {
  if (!inTauri) return import.meta.env.DEV ? "D:\\Exports" : null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const r = await open({ directory: true, multiple: false, defaultPath: defaultPath ?? undefined });
  return typeof r === "string" ? r : null;
}

export type DropState = "over" | "leave" | { path: string };

/** Native file drops (Tauri gives real paths; the browser cannot, so dev only shows hover). */
export async function onFileDrop(cb: (s: DropState) => void): Promise<() => void> {
  if (!inTauri) return () => {};
  const { getCurrentWebview } = await import("@tauri-apps/api/webview");
  return getCurrentWebview().onDragDropEvent((e) => {
    const p = e.payload;
    if (p.type === "enter" || p.type === "over") cb("over");
    else if (p.type === "leave") cb("leave");
    else if (p.type === "drop") {
      cb("leave");
      const path = p.paths.find((x) => VIDEO_EXTENSIONS.includes(x.split(".").pop()?.toLowerCase() ?? "")) ?? p.paths[0];
      if (path) cb({ path });
    }
  });
}
