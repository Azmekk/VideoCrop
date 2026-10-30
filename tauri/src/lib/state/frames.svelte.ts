// Filmstrip thumbnail cache: blob URLs keyed by time, at most 2 requests in flight,
// always working on the most recently requested (visible) set.

import { frameSrc } from "../api";

const MAX_CONCURRENT = 2;
const MAX_ENTRIES = 600;
const W = 160;

const cache = new Map<string, string>();
const inflight = new Set<string>();
let queue: number[] = [];
let base = "";
let active = 0;
let generation = 0;

/** Bumped whenever a thumbnail lands, so components re-read `thumb()`. */
export const thumbs = $state({ version: 0 });

const key = (t: number) => t.toFixed(3);

export function thumb(t: number): string | undefined {
  return cache.get(key(t));
}

export function resetThumbs(frameUrl: string) {
  for (const u of cache.values()) URL.revokeObjectURL(u);
  cache.clear();
  inflight.clear();
  queue = [];
  base = frameUrl;
  generation++;
  thumbs.version++;
}

/** Replace the wanted set (ordered by priority). Already cached or in-flight times are skipped. */
export function wantThumbs(times: number[]) {
  queue = times.filter((t) => !cache.has(key(t)) && !inflight.has(key(t)));
  pump();
}

function pump() {
  while (active < MAX_CONCURRENT && queue.length) {
    const t = queue.shift()!;
    const k = key(t);
    if (cache.has(k) || inflight.has(k)) continue;
    active++;
    inflight.add(k);
    const gen = generation;
    fetch(frameSrc(base, t, W, true))
      .then((r) => (r.ok ? r.blob() : Promise.reject(r.status)))
      .then((b) => {
        if (gen !== generation) return;
        cache.set(k, URL.createObjectURL(b));
        if (cache.size > MAX_ENTRIES) {
          const oldest = cache.keys().next().value!;
          URL.revokeObjectURL(cache.get(oldest)!);
          cache.delete(oldest);
        }
        thumbs.version++;
      })
      .catch(() => {})
      .finally(() => {
        inflight.delete(k);
        active--;
        pump();
      });
  }
}
