// Cross-store actions: opening a file, applying a crop, building the JobDraft and planning.

import { api } from "../api";
import { pickVideo } from "../platform";
import type { CropSpec, JobDraft } from "../types";
import { editor } from "./editor.svelte";
import { cut } from "./cut.svelte";
import { crop } from "./crop.svelte";
import { resize } from "./resize.svelte";
import { exporter } from "./export.svelte";
import { encoder } from "./encode.svelte";
import { settings } from "./settings.svelte";
import { toasts } from "./toast.svelte";

export function buildDraft(): JobDraft {
  return {
    cut: cut.toSpec(),
    crop: crop.spec ? { ...crop.spec } : null,
    resize: resize.toSpec(),
    compression: exporter.choice,
    mode: exporter.mode,
    outputDir: exporter.outputDir,
  };
}

export async function openVideo(path: string) {
  if (encoder.busy || editor.opening) return;
  editor.opening = true;
  editor.openError = null;
  try {
    const v = await api.openVideo(path);
    editor.player?.pause();
    editor.mode = "edit";
    editor.previewCropped = false;
    editor.time = 0;
    editor.scrubTime = null;
    cut.accurate = false;
    cut.reset(v.info.durationSec);
    crop.spec = null;
    crop.aspect = "free";
    editor.video = v;
    resize.reset();
    exporter.plan = null;
    encoder.dismiss();
    settings.update({ recentFiles: [path, ...settings.value.recentFiles.filter((p) => p !== path)].slice(0, 8) });
  } catch (e) {
    editor.openError = String(e);
  } finally {
    editor.opening = false;
  }
}

export async function closeVideo() {
  if (encoder.busy) return;
  editor.player?.pause();
  editor.video = null;
  editor.mode = "edit";
  encoder.dismiss();
  exporter.plan = null;
  await api.closeVideo().catch(() => {});
}

export function applyCrop(c: CropSpec | null) {
  crop.spec = c ? crop.finalize(c) : null;
  if (!crop.spec) editor.previewCropped = false;
  const msg = resize.syncToInput();
  if (msg) toasts.show(msg);
}

let planTimer: ReturnType<typeof setTimeout> | undefined;
let planSeq = 0;

/** Debounced plan_job; the backend is the single source of truth for the summary. */
export function schedulePlan(draft: JobDraft, delay = 120) {
  clearTimeout(planTimer);
  planTimer = setTimeout(async () => {
    const seq = ++planSeq;
    try {
      const p = await api.planJob(draft);
      if (seq === planSeq) { exporter.plan = p; exporter.planError = null; }
    } catch (e) {
      if (seq === planSeq) exporter.planError = String(e);
    }
  }, delay);
}

/** Open while the user is asked to confirm a size target that can't look good (plan.sizeAlert). */
export const sizeConfirm = $state({ open: false, message: "" });

/** Starts the encode. A plan with a size alert asks first; `confirmed` skips the question. */
export function process(confirmed = false) {
  if (!editor.video || encoder.busy) return;
  if (editor.mode === "crop" || !exporter.ready) return;
  const alert = exporter.plan?.sizeAlert;
  if (alert && !confirmed) {
    sizeConfirm.message = alert;
    sizeConfirm.open = true;
    return;
  }
  sizeConfirm.open = false;
  editor.player?.pause();
  void encoder.start(buildDraft());
}

export function enterCropMode() {
  if (!editor.video || encoder.busy) return;
  editor.player?.pause();
  crop.beginEdit();
  editor.mode = "crop";
}

export function exitCropMode(apply: boolean) {
  if (apply) applyCrop(crop.normalizedDraft());
  editor.mode = "edit";
}

/** File picker -> open. */
export async function chooseAndOpen() {
  if (encoder.busy) return;
  const p = await pickVideo();
  if (p) await openVideo(p);
}
