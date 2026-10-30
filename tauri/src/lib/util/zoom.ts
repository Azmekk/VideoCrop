// Timeline viewport math: a visible window [start, start+span] over [0, duration].

export interface View { start: number; span: number }

export function clampView(v: View, duration: number, minSpan: number): View {
  const span = Math.min(duration, Math.max(minSpan, v.span));
  const start = Math.min(duration - span, Math.max(0, v.start));
  return { start, span };
}

/** Zoom by `factor` (<1 zooms in) keeping time `anchor` under the same pixel. */
export function zoomAround(v: View, factor: number, anchor: number, duration: number, minSpan: number): View {
  const span = Math.min(duration, Math.max(minSpan, v.span * factor));
  const rel = v.span > 0 ? (anchor - v.start) / v.span : 0.5;
  return clampView({ start: anchor - rel * span, span }, duration, minSpan);
}

export const pan = (v: View, dt: number, duration: number, minSpan: number): View =>
  clampView({ start: v.start + dt, span: v.span }, duration, minSpan);

/** View that frames [a, b] with a little breathing room. */
export function frameRange(a: number, b: number, duration: number, minSpan: number, pad = 0.06): View {
  const len = Math.max(b - a, minSpan);
  const span = len * (1 + pad * 2);
  return clampView({ start: a - len * pad, span }, duration, minSpan);
}

export const timeToX = (t: number, v: View, width: number) => ((t - v.start) / v.span) * width;
export const xToTime = (x: number, v: View, width: number) => v.start + (x / width) * v.span;

const STEPS = [
  0.001, 0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5,
  1, 2, 5, 10, 15, 30, 60, 120, 300, 600, 900, 1800, 3600, 7200,
];

/**
 * Major tick step so labels are at least `minPx` apart. When fps is known and steps get
 * below one frame, snaps to whole frames.
 */
export function tickStep(span: number, width: number, minPx = 84, fps = 0): number {
  const minStep = (span / Math.max(1, width)) * minPx;
  if (fps > 0 && minStep < 1) {
    const frame = 1 / fps;
    for (const n of [1, 2, 5, 10, 15]) if (n * frame >= minStep) return n * frame;
  }
  return STEPS.find((s) => s >= minStep) ?? 7200;
}

export function ticks(v: View, step: number): number[] {
  const out: number[] = [];
  const first = Math.ceil(v.start / step - 1e-9) * step;
  for (let t = first; t <= v.start + v.span + 1e-9 && out.length < 400; t += step) out.push(+t.toFixed(6));
  return out;
}

/** Minor tick spacing for a major step (whole frames when the major step is a few frames). */
export function minorStep(step: number, fps: number): number {
  const frames = fps > 0 ? step * fps : Infinity;
  if (frames < 1.5) return step;
  if (frames <= 15 && Math.abs(frames - Math.round(frames)) < 1e-6) {
    const f = Math.round(frames);
    return (f <= 5 ? 1 : f % 5 === 0 ? 5 : 1) / fps;
  }
  const special: Record<number, number> = { 15: 3, 30: 6, 60: 6, 900: 3, 1800: 6, 3600: 6 };
  if (special[step]) return step / special[step];
  const lead = Math.round(step / 10 ** Math.floor(Math.log10(step) + 1e-9));
  return step / (lead === 2 ? 4 : 5);
}
