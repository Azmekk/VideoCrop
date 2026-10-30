import { clampView, pan, zoomAround, type View } from "../util/zoom";
import { editor } from "./editor.svelte";
import { cut } from "./cut.svelte";

/** Visible window of the timeline. Kept in a store so keyboard shortcuts can zoom. */
class TimelineStore {
  view = $state<View>({ start: 0, span: 1 });
  /** Set by "Zoom to cut": the view can't leave this range until "Show all". */
  focus = $state<{ lo: number; hi: number } | null>(null);

  get duration() { return editor.info?.durationSec ?? 1; }
  get lo() { return this.focus?.lo ?? 0; }
  get hi() { return this.focus?.hi ?? this.duration; }
  /** Few frames across the whole width at max zoom. */
  get minSpan() { return Math.min(this.hi - this.lo, 6 / editor.fps); }
  get zoomed() { return this.view.span < this.duration * 0.999; }
  /** False when zooming out further is blocked (whole file, or the whole focused cut). */
  get canZoomOut() { return this.view.span < (this.hi - this.lo) * 0.999; }

  fitAll() { this.focus = null; this.view = { start: 0, span: this.duration }; }

  set(v: View) { this.view = clampView(v, this.hi, this.minSpan, this.lo); }

  zoomBy(factor: number, anchor?: number) {
    const a = anchor ?? editor.displayTime;
    this.view = zoomAround(this.view, factor, a, this.hi, this.minSpan, this.lo);
  }

  panBy(dt: number) { this.view = pan(this.view, dt, this.hi, this.minSpan, this.lo); }

  zoomToCut() {
    if (cut.isFull) { this.fitAll(); return; }
    this.focus = { lo: cut.start, hi: cut.end };
    this.view = { start: cut.start, span: cut.end - cut.start };
  }

  /** Pages the view so `t` stays visible (used during playback). */
  follow(t: number) {
    const v = this.view;
    if (!this.zoomed) return;
    if (t < v.start || t > v.start + v.span) this.set({ start: t - v.span * 0.1, span: v.span });
  }
}

export const timeline = new TimelineStore();
