import { clampView, frameRange, pan, zoomAround, type View } from "../util/zoom";
import { editor } from "./editor.svelte";
import { cut } from "./cut.svelte";

/** Visible window of the timeline. Kept in a store so keyboard shortcuts can zoom. */
class TimelineStore {
  view = $state<View>({ start: 0, span: 1 });

  get duration() { return editor.info?.durationSec ?? 1; }
  /** Few frames across the whole width at max zoom. */
  get minSpan() { return Math.min(this.duration, 6 / editor.fps); }
  get zoomed() { return this.view.span < this.duration * 0.999; }

  fitAll() { this.view = { start: 0, span: this.duration }; }

  set(v: View) { this.view = clampView(v, this.duration, this.minSpan); }

  zoomBy(factor: number, anchor?: number) {
    const a = anchor ?? editor.displayTime;
    this.view = zoomAround(this.view, factor, a, this.duration, this.minSpan);
  }

  panBy(dt: number) { this.view = pan(this.view, dt, this.duration, this.minSpan); }

  zoomToCut() { this.view = frameRange(cut.start, cut.end, this.duration, this.minSpan); }

  /** Pages the view so `t` stays visible (used during playback). */
  follow(t: number) {
    const v = this.view;
    if (!this.zoomed) return;
    if (t < v.start || t > v.start + v.span) this.set({ start: t - v.span * 0.1, span: v.span });
  }
}

export const timeline = new TimelineStore();
