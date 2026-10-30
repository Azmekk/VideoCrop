// Aspect / crop / resize math. CropSpec is always in SOURCE pixels.

import type { CropSpec, VideoInfo } from "../types";

export const evenFloor = (n: number) => Math.max(0, Math.floor(n / 2) * 2);
export const evenRound = (n: number) => Math.max(0, Math.round(n / 2) * 2);

const gcd = (a: number, b: number): number => (b ? gcd(b, a % b) : a);

const NAMED: [number, string][] = [
  [16 / 9, "16:9"], [9 / 16, "9:16"], [4 / 3, "4:3"], [3 / 4, "3:4"], [1, "1:1"],
  [4 / 5, "4:5"], [5 / 4, "5:4"], [21 / 9, "21:9"], [64 / 27, "21:9"], [3 / 2, "3:2"],
  [2 / 3, "2:3"], [16 / 10, "16:10"], [2.39, "2.39:1"], [1.85, "1.85:1"],
];

/** `16:9` for common ratios (±1 %), else reduced `W:H` or a decimal `1.78:1`. */
export function aspectLabel(w: number, h: number): string {
  if (!w || !h) return "";
  const r = w / h;
  for (const [v, name] of NAMED) if (Math.abs(r - v) / v < 0.01) return name;
  const g = gcd(Math.round(w), Math.round(h));
  const a = Math.round(w) / g, b = Math.round(h) / g;
  return a <= 32 && b <= 32 ? `${a}:${b}` : `${r.toFixed(2)}:1`;
}

/** Display width accounting for anamorphic SAR. */
export function displaySize(info: Pick<VideoInfo, "width" | "height" | "sarNum" | "sarDen">) {
  const sar = info.sarNum > 0 && info.sarDen > 0 ? info.sarNum / info.sarDen : 1;
  return { width: Math.round(info.width * sar), height: info.height };
}

export const fullCrop = (srcW: number, srcH: number): CropSpec => ({
  x: 0, y: 0, width: evenFloor(srcW), height: evenFloor(srcH),
});

export const isFullFrame = (c: CropSpec | null, srcW: number, srcH: number) =>
  !c || (c.x === 0 && c.y === 0 && c.width >= evenFloor(srcW) && c.height >= evenFloor(srcH));

/** Even-rounds and clamps a crop into the source frame (x,y even; w,h even ≥ 2). */
export function normalizeCrop(c: CropSpec, srcW: number, srcH: number): CropSpec {
  const maxW = evenFloor(srcW), maxH = evenFloor(srcH);
  const width = Math.min(maxW, Math.max(2, evenRound(c.width)));
  const height = Math.min(maxH, Math.max(2, evenRound(c.height)));
  const x = Math.min(evenFloor(srcW - width), Math.max(0, evenRound(c.x)));
  const y = Math.min(evenFloor(srcH - height), Math.max(0, evenRound(c.y)));
  return { x, y, width, height };
}

/** Largest centered rect of `ratio` (w/h, in source pixels) inside the given rect. */
export function fitAspect(c: CropSpec, ratio: number): CropSpec {
  let w = c.width, h = w / ratio;
  if (h > c.height) { h = c.height; w = h * ratio; }
  return { x: c.x + (c.width - w) / 2, y: c.y + (c.height - h) / 2, width: w, height: h };
}

/** Derived height for an aspect-locked width (even). */
export const lockedHeight = (w: number, inW: number, inH: number) => Math.max(2, evenRound((w * inH) / inW));
export const lockedWidth = (h: number, inW: number, inH: number) => Math.max(2, evenRound((h * inW) / inH));

export type Handle = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw" | "move";

/**
 * Applies a drag delta (source px) to a rect for the given handle. `ratio` (w/h) constrains the
 * shape when set. Returns an un-normalized rect clamped to the frame.
 */
export function dragRect(
  start: CropSpec, h: Handle, dx: number, dy: number, srcW: number, srcH: number, ratio: number | null,
  minSize = 16,
): CropSpec {
  if (h === "move") {
    const x = Math.min(srcW - start.width, Math.max(0, start.x + dx));
    const y = Math.min(srcH - start.height, Math.max(0, start.y + dy));
    return { ...start, x, y };
  }
  let l = start.x, t = start.y, r = start.x + start.width, b = start.y + start.height;
  if (h.includes("w")) l = Math.min(r - minSize, Math.max(0, l + dx));
  if (h.includes("e")) r = Math.max(l + minSize, Math.min(srcW, r + dx));
  if (h.includes("n")) t = Math.min(b - minSize, Math.max(0, t + dy));
  if (h.includes("s")) b = Math.max(t + minSize, Math.min(srcH, b + dy));
  if (ratio) {
    let w = r - l, ht = b - t;
    const horiz = h === "e" || h === "w";
    const vert = h === "n" || h === "s";
    if (horiz) ht = w / ratio;
    else if (vert) w = ht * ratio;
    else if (w / ht > ratio) w = ht * ratio;
    else ht = w / ratio;
    // Anchor: the side opposite the dragged handle stays put; edges stay centered.
    const cx = (start.x * 2 + start.width) / 2, cy = (start.y * 2 + start.height) / 2;
    let nl = h.includes("w") ? r - w : h.includes("e") ? l : cx - w / 2;
    let nt = h.includes("n") ? b - ht : h.includes("s") ? t : cy - ht / 2;
    // Shrink to fit the frame if needed.
    const fit = Math.min(1, srcW / w, srcH / ht,
      h.includes("w") ? r / w : h.includes("e") ? (srcW - l) / w : 1,
      h.includes("n") ? b / ht : h.includes("s") ? (srcH - t) / ht : 1);
    if (fit < 1) {
      w *= fit; ht *= fit;
      nl = h.includes("w") ? r - w : h.includes("e") ? l : cx - w / 2;
      nt = h.includes("n") ? b - ht : h.includes("s") ? t : cy - ht / 2;
    }
    nl = Math.min(srcW - w, Math.max(0, nl));
    nt = Math.min(srcH - ht, Math.max(0, nt));
    return { x: nl, y: nt, width: w, height: ht };
  }
  return { x: l, y: t, width: r - l, height: b - t };
}
