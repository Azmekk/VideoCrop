// iOS-style fine scrubbing: the further the pointer moves vertically from the track, the
// slower the playhead follows horizontal motion.

export interface ScrubRate { factor: number; label: string; frameSnap: boolean }

const LEVELS: [number, ScrubRate][] = [
  [48, { factor: 1, label: "Full speed", frameSnap: false }],
  [110, { factor: 0.5, label: "½× scrub", frameSnap: false }],
  [180, { factor: 0.25, label: "¼× scrub", frameSnap: false }],
  [260, { factor: 0.125, label: "⅛× scrub", frameSnap: false }],
  [Infinity, { factor: 0.04, label: "Frame precise", frameSnap: true }],
];

/** `dy` = vertical distance (px) from the track; `alt` forces at least fine mode. */
export function scrubRate(dy: number, alt = false): ScrubRate {
  const d = Math.abs(dy);
  let rate = LEVELS.find(([lim]) => d < lim)![1];
  if (alt && rate.factor > 0.125) rate = LEVELS[3][1];
  return rate;
}

/**
 * Integrates pointer motion into a time value: `prev` + horizontal delta × seconds-per-px × factor.
 * Using relative motion (not absolute x) is what makes slow modes possible.
 */
export const scrubStep = (prev: number, dxPx: number, secPerPx: number, rate: ScrubRate) =>
  prev + dxPx * secPerPx * rate.factor;

export const snapToFrame = (t: number, fps: number) => (fps > 0 ? Math.round(t * fps) / fps : t);
