import type { ResizeSpec } from "../types";
import { editor } from "./editor.svelte";
import { crop } from "./crop.svelte";
import { evenRound, lockedHeight, lockedWidth } from "../util/geometry";

export const HEIGHT_PRESETS = [2160, 1440, 1080, 720, 480, 360];
export const PERCENT_PRESETS = [100, 75, 50, 33, 25];

/** Resize of the POST-CROP frame. */
class ResizeStore {
  enabled = $state(false);
  width = $state(0);
  height = $state(0);
  locked = $state(true);

  /** Post-crop input dimensions (source pixels). */
  get inW() { return crop.spec?.width ?? editor.info?.width ?? 0; }
  get inH() { return crop.spec?.height ?? editor.info?.height ?? 0; }

  get mismatch() {
    if (!this.enabled || !this.inW || !this.inH || !this.height) return false;
    const a = this.width / this.height, b = this.inW / this.inH;
    return Math.abs(a - b) / b > 0.01;
  }

  get isIdentity() { return this.width === evenRound(this.inW) && this.height === evenRound(this.inH); }

  reset() {
    this.enabled = false;
    this.locked = true;
    this.width = evenRound(this.inW);
    this.height = evenRound(this.inH);
  }

  setWidth(w: number) {
    this.width = Math.max(2, evenRound(w));
    if (this.locked) this.height = lockedHeight(this.width, this.inW, this.inH);
  }

  setHeight(h: number) {
    this.height = Math.max(2, evenRound(h));
    if (this.locked) this.width = lockedWidth(this.height, this.inW, this.inH);
  }

  setLocked(on: boolean) {
    this.locked = on;
    if (on) this.height = lockedHeight(this.width, this.inW, this.inH);
  }

  presetHeight(h: number) {
    this.enabled = true;
    this.height = evenRound(h);
    this.width = lockedWidth(this.height, this.inW, this.inH);
  }

  percent(p: number) {
    this.enabled = true;
    this.width = Math.max(2, evenRound((this.inW * p) / 100));
    this.height = Math.max(2, evenRound((this.inH * p) / 100));
  }

  /**
   * Called after the crop changes. Returns a toast message when dimensions were recomputed.
   * Locked: keep width (capped at the new input) and derive height. Disabled: track the input.
   */
  syncToInput(): string | null {
    if (!this.enabled) { this.width = evenRound(this.inW); this.height = evenRound(this.inH); return null; }
    if (!this.locked) return null;
    const w = Math.min(this.width, evenRound(this.inW));
    const h = lockedHeight(w, this.inW, this.inH);
    if (w === this.width && h === this.height) return null;
    this.width = w;
    this.height = h;
    return `Resize updated to ${w}×${h} to match new aspect`;
  }

  toSpec(): ResizeSpec | null {
    if (!this.enabled || this.isIdentity) return null;
    return { width: this.width, height: this.height, aspectLocked: this.locked };
  }
}

export const resize = new ResizeStore();
