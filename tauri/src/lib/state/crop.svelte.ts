import type { CropSpec } from "../types";
import { editor } from "./editor.svelte";
import { fitAspect, fullCrop, isFullFrame, normalizeCrop, displaySize } from "../util/geometry";

export type AspectId = "free" | "original" | "16:9" | "4:3" | "1:1" | "9:16" | "4:5";
export const ASPECTS: { id: AspectId; label: string }[] = [
  { id: "free", label: "Free" }, { id: "original", label: "Original" }, { id: "16:9", label: "16:9" },
  { id: "4:3", label: "4:3" }, { id: "1:1", label: "1:1" }, { id: "9:16", label: "9:16" }, { id: "4:5", label: "4:5" },
];

/** Applied crop (`spec`, null = full frame) plus the in-progress draft while editing. */
class CropStore {
  spec = $state<CropSpec | null>(null);
  draft = $state<CropSpec>({ x: 0, y: 0, width: 2, height: 2 });
  aspect = $state<AspectId>("free");

  get srcW() { return editor.info?.width ?? 0; }
  get srcH() { return editor.info?.height ?? 0; }

  /** Display aspect ratio (w/h) for an aspect chip, converted to SOURCE-pixel ratio via SAR. */
  ratioFor(id: AspectId): number | null {
    const info = editor.info;
    if (!info || id === "free") return null;
    const sar = info.sarNum > 0 && info.sarDen > 0 ? info.sarNum / info.sarDen : 1;
    if (id === "original") return info.width / info.height;
    const [a, b] = id.split(":").map(Number);
    return a / b / sar;
  }

  /** Display-space size of the applied crop (accounts for SAR). */
  get displayDims() {
    const info = editor.info;
    if (!info) return { width: 0, height: 0 };
    return this.displayDimsFor(this.spec ?? fullCrop(info.width, info.height));
  }

  displayDimsFor(c: CropSpec) {
    const info = editor.info;
    if (!info) return { width: c.width, height: c.height };
    return displaySize({ ...info, width: c.width, height: c.height });
  }

  beginEdit() {
    this.draft = this.spec ? { ...this.spec } : fullCrop(this.srcW, this.srcH);
  }

  setDraft(c: CropSpec) {
    this.draft = c;
  }

  /** Snap the draft to even, clamped source pixels. */
  normalizedDraft(): CropSpec {
    return normalizeCrop(this.draft, this.srcW, this.srcH);
  }

  chooseAspect(id: AspectId) {
    this.aspect = id;
    const r = this.ratioFor(id);
    if (r) this.draft = normalizeCrop(fitAspect(this.draft, r), this.srcW, this.srcH);
  }

  resetDraft() {
    this.aspect = "free";
    this.draft = fullCrop(this.srcW, this.srcH);
  }

  /** Returns the normalized spec to store (null when it is the full frame). */
  finalize(c: CropSpec): CropSpec | null {
    const n = normalizeCrop(c, this.srcW, this.srcH);
    return isFullFrame(n, this.srcW, this.srcH) ? null : n;
  }
}

export const crop = new CropStore();
