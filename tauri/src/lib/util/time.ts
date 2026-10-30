// Time formatting / parsing for timecode readouts and the start/end fields.

const pad = (n: number, w = 2) => String(n).padStart(w, "0");

/** `MM:SS.mmm`, or `H:MM:SS.mmm` when `withHours` (or the value itself is ≥ 1 h). */
export function formatTime(sec: number, withHours = false): string {
  if (!Number.isFinite(sec) || sec < 0) sec = 0;
  const totalMs = Math.round(sec * 1000);
  const ms = totalMs % 1000;
  const s = Math.floor(totalMs / 1000) % 60;
  const m = Math.floor(totalMs / 60000) % 60;
  const h = Math.floor(totalMs / 3600000);
  const tail = `${pad(s)}.${pad(ms, 3)}`;
  return h > 0 || withHours ? `${h}:${pad(m)}:${tail}` : `${pad(m)}:${tail}`;
}

/** Compact label for ruler ticks, precision chosen from the tick step. */
export function formatTick(sec: number, step: number): string {
  const h = Math.floor(sec / 3600);
  const m = Math.floor(sec / 60) % 60;
  const s = sec - Math.floor(sec / 60) * 60;
  const decimals = step >= 1 ? 0 : step >= 0.1 ? 1 : step >= 0.01 ? 2 : 3;
  let ss = s.toFixed(decimals);
  if (Number(ss) >= 60) ss = (0).toFixed(decimals); // rounding guard
  const [int, frac] = ss.split(".");
  const sPart = `${pad(Number(int))}${frac ? "." + frac : ""}`;
  return h > 0 ? `${h}:${pad(m)}:${sPart}` : `${m}:${sPart}`;
}

/** Human duration: `12.18 s`, `4 min 18 s`, `1 h 02 min`. */
export function formatDuration(sec: number): string {
  if (sec < 60) return `${sec.toFixed(sec < 10 ? 2 : 1)} s`;
  const m = Math.floor(sec / 60);
  if (m < 60) return `${m} min ${pad(Math.floor(sec % 60))} s`;
  return `${Math.floor(m / 60)} h ${pad(m % 60)} min`;
}

/** Accepts `H:MM:SS.mmm`, `MM:SS.mmm`, `SS.mmm`, plain seconds, or `,` as decimal mark. */
export function parseTime(input: string): number | null {
  const s = input.trim().replace(",", ".");
  if (!s) return null;
  if (!/^\d+(:\d+){0,2}(\.\d*)?$/.test(s) && !/^\d*\.\d+$/.test(s)) return null;
  const parts = s.split(":");
  let total = 0;
  for (let i = 0; i < parts.length; i++) {
    const v = Number(parts[i]);
    if (!Number.isFinite(v)) return null;
    // Non-leading minute/second fields must be < 60.
    if (i > 0 && v >= 60) return null;
    total = total * 60 + v;
  }
  return total;
}

/**
 * Step size for the segment under the caret in a formatted time string
 * (`H:MM:SS.mmm` / `MM:SS.mmm` / plain seconds).
 */
export function segmentStep(text: string, caret: number): number {
  const dot = text.indexOf(".");
  if (dot >= 0 && caret > dot) return 0.001 * 10 ** Math.max(0, 3 - (caret - dot)) || 0.001;
  const head = dot >= 0 ? text.slice(0, dot) : text;
  const colonsBefore = (head.slice(0, caret).match(/:/g) ?? []).length;
  const totalColons = (head.match(/:/g) ?? []).length;
  const fromRight = totalColons - colonsBefore; // 0 = seconds, 1 = minutes, 2 = hours
  return [1, 60, 3600][Math.min(fromRight, 2)];
}

export const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));
