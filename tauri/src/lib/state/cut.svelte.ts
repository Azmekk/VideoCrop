import type { CutSpec } from "../types";
import { clamp } from "../util/time";

export const MIN_CUT = 0.1;

/** Cut range in source seconds. `toSpec()` is null when the full file is kept. */
class CutStore {
  duration = $state(0);
  start = $state(0);
  end = $state(0);
  accurate = $state(false);

  get length() { return this.end - this.start; }
  get isFull() { return this.start <= 0.0005 && this.end >= this.duration - 0.0005; }

  reset(duration = this.duration) {
    this.duration = duration;
    this.start = 0;
    this.end = duration;
  }

  setStart(t: number) {
    this.start = clamp(t, 0, Math.max(0, this.end - MIN_CUT));
  }

  setEnd(t: number) {
    this.end = clamp(t, Math.min(this.duration, this.start + MIN_CUT), this.duration);
  }

  /** Clamp a time into the playable range [start, end]. */
  clampTime(t: number) {
    return clamp(t, this.start, this.end);
  }

  toSpec(): CutSpec | null {
    if (this.isFull) return null;
    return { startSec: +this.start.toFixed(3), endSec: +this.end.toFixed(3), accurate: this.accurate };
  }
}

export const cut = new CutStore();
