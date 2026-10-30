<script lang="ts">
  // Lazy thumbnails across the visible window. Tile times sit on a power-of-two frame grid so
  // panning and zooming reuse cached frames.
  import { thumb, thumbs, wantThumbs } from "../../lib/state/frames.svelte";
  import { timeToX, type View } from "../../lib/util/zoom";

  let { view, width, height, duration, fps, aspect }: {
    view: View; width: number; height: number; duration: number; fps: number; aspect: number;
  } = $props();

  const tiles = $derived.by(() => {
    if (width <= 0 || height <= 0) return [];
    const tileW = Math.max(24, height * aspect);
    const raw = (view.span * tileW) / width;
    const frame = 1 / (fps || 30);
    const step = Math.pow(2, Math.ceil(Math.log2(Math.max(1, raw / frame)))) * frame;
    const first = Math.max(0, Math.floor(view.start / step));
    const last = Math.min(Math.ceil(duration / step), Math.ceil((view.start + view.span) / step));
    const out: { t: number; x: number; w: number }[] = [];
    for (let k = first; k <= last && out.length < 200; k++) {
      const t0 = k * step;
      if (t0 >= duration) break;
      const x = timeToX(t0, view, width);
      const w = timeToX(Math.min(duration, t0 + step), view, width) - x;
      const t = +Math.min(t0 + step / 2, Math.max(0, duration - 0.05)).toFixed(3);
      if (out.length && out[out.length - 1].t === t) break;
      out.push({ t, x, w });
    }
    return out;
  });

  // `version` is read only to re-render when a thumbnail lands.
  const srcFor = (t: number, _version: number) => thumb(t);

  // Request visible tiles, center-out so the area under attention fills first.
  $effect(() => {
    const ts = tiles;
    const mid = width / 2;
    wantThumbs([...ts].sort((a, b) => Math.abs(a.x + a.w / 2 - mid) - Math.abs(b.x + b.w / 2 - mid)).map((t) => t.t));
  });
</script>

<div class="strip" aria-hidden="true">
  {#each tiles as t (t.t)}
    {@const src = srcFor(t.t, thumbs.version)}
    <div class="tile" style:left="{t.x}px" style:width="{t.w + 0.5}px">
      {#if src}<img {src} alt="" draggable="false" />{/if}
    </div>
  {/each}
</div>

<style>
  .strip { position: absolute; inset: 0; overflow: hidden; pointer-events: none; }
  .tile { position: absolute; top: 0; bottom: 0; overflow: hidden; background: var(--track); border-right: 1px solid rgb(0 0 0 / .25); }
  img { width: 100%; height: 100%; object-fit: cover; display: block; animation: fade var(--med) ease; }
  @keyframes fade { from { opacity: 0; } }
</style>
