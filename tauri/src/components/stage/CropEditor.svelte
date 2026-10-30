<script lang="ts">
  // Crop overlay over an exact still of the paused frame. The rect lives in SOURCE pixels;
  // the overlay maps it to the box proportionally on each axis (SAR-safe).
  import { frameSrc } from "../../lib/api";
  import { editor } from "../../lib/state/editor.svelte";
  import { crop } from "../../lib/state/crop.svelte";
  import { exitCropMode } from "../../lib/state/pipeline.svelte";
  import { dragRect, normalizeCrop, type Handle } from "../../lib/util/geometry";
  import type { CropSpec } from "../../lib/types";

  let { boxW, boxH }: { boxW: number; boxH: number } = $props();

  const info = $derived(editor.info!);
  const sx = $derived(info.width / Math.max(1, boxW));
  const sy = $derived(info.height / Math.max(1, boxH));

  // Exact still at the playhead (native size). If it fails, the paused video shows through.
  const still = $derived(editor.video ? frameSrc(editor.video.frameUrl, editor.time, undefined, false) : "");
  let stillOk = $state(false);

  let dragging = $state<Handle | null>(null);
  let start: CropSpec | null = null;
  let origin = { x: 0, y: 0 };
  let rectEl = $state<HTMLDivElement>();

  const d = $derived(crop.draft);
  const pct = $derived({
    l: (d.x / info.width) * 100, t: (d.y / info.height) * 100,
    w: (d.width / info.width) * 100, h: (d.height / info.height) * 100,
  });
  const shown = $derived(normalizeCrop(d, info.width, info.height));

  $effect(() => { rectEl?.focus({ preventScroll: true }); });

  const HANDLES: Handle[] = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];

  function down(e: PointerEvent, h: Handle) {
    if (e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    start = { ...crop.draft };
    origin = { x: e.clientX, y: e.clientY };
    dragging = h;
    rectEl?.focus({ preventScroll: true });
  }

  /** Pointer down on the dimmed area starts a new rectangle from that point. */
  function downOutside(e: PointerEvent) {
    if (e.button !== 0) return;
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = (e.clientX - r.left) * sx, y = (e.clientY - r.top) * sy;
    crop.setDraft({ x, y, width: 16, height: 16 });
    down(e, "se");
  }

  function move(e: PointerEvent) {
    if (!dragging || !start) return;
    const dx = (e.clientX - origin.x) * sx, dy = (e.clientY - origin.y) * sy;
    crop.setDraft(dragRect(start, dragging, dx, dy, info.width, info.height, crop.ratioFor(crop.aspect)));
  }

  function up() {
    if (!dragging) return;
    dragging = null;
    start = null;
    crop.setDraft(crop.normalizedDraft());
  }

  function key(e: KeyboardEvent) {
    if (e.key === "Enter") { e.preventDefault(); exitCropMode(true); return; }
    if (e.key === "Escape") { e.preventDefault(); exitCropMode(false); return; }
    const step = e.shiftKey ? 20 : 2;
    const dir: Record<string, [number, number]> = { ArrowLeft: [-step, 0], ArrowRight: [step, 0], ArrowUp: [0, -step], ArrowDown: [0, step] };
    const v = dir[e.key];
    if (!v) return;
    e.preventDefault();
    const r = crop.ratioFor(crop.aspect);
    // Ctrl+arrows resize from the bottom-right corner; plain arrows move.
    const next = e.ctrlKey
      ? dragRect(crop.draft, v[0] ? "e" : "s", v[0], v[1], info.width, info.height, r)
      : dragRect(crop.draft, "move", v[0], v[1], info.width, info.height, null);
    crop.setDraft(normalizeCrop(next, info.width, info.height));
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" class:dragging={!!dragging} onpointerdown={downOutside} onpointermove={move} onpointerup={up} onpointercancel={up}>
  {#if still}
    <img class="still" class:ok={stillOk} src={still} alt="" draggable="false"
      onload={() => (stillOk = true)} onerror={() => (stillOk = false)} />
  {/if}

  <div class="shade" aria-hidden="true">
    <div class="hole" style:left="{pct.l}%" style:top="{pct.t}%" style:width="{pct.w}%" style:height="{pct.h}%"></div>
  </div>

  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div
    class="rect" bind:this={rectEl} tabindex="0" role="group"
    aria-label="Crop area {shown.width}×{shown.height} at {shown.x},{shown.y}. Arrows move, Ctrl+arrows resize, Enter applies, Escape cancels."
    style:left="{pct.l}%" style:top="{pct.t}%" style:width="{pct.w}%" style:height="{pct.h}%"
    onpointerdown={(e) => down(e, "move")} onkeydown={key}
  >
    <div class="thirds" aria-hidden="true"><i></i><i></i><i></i><i></i></div>
    <span class="dims tnum">{shown.width}×{shown.height}</span>
    {#each HANDLES as h (h)}
      <span class="h {h}" onpointerdown={(e) => down(e, h)} aria-hidden="true"></span>
    {/each}
  </div>
</div>

<style>
  .overlay { position: absolute; inset: 0; cursor: crosshair; touch-action: none; }
  .still { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: fill; opacity: 0; pointer-events: none; }
  .still.ok { opacity: 1; }
  .rect {
    position: absolute; cursor: move; outline: none;
    border: 1px solid #fff;
  }
  .shade { position: absolute; inset: 0; overflow: hidden; pointer-events: none; }
  .hole { position: absolute; box-shadow: 0 0 0 9999px rgb(0 0 0 / .6); }
  .rect:focus-visible { border-color: var(--accent); }
  .thirds { position: absolute; inset: 0; opacity: 0; transition: opacity var(--fast); pointer-events: none; }
  .dragging .thirds { opacity: 1; }
  .thirds i { position: absolute; background: rgb(255 255 255 / .45); }
  .thirds i:nth-child(1) { left: 33.333%; top: 0; bottom: 0; width: 1px; }
  .thirds i:nth-child(2) { left: 66.666%; top: 0; bottom: 0; width: 1px; }
  .thirds i:nth-child(3) { top: 33.333%; left: 0; right: 0; height: 1px; }
  .thirds i:nth-child(4) { top: 66.666%; left: 0; right: 0; height: 1px; }
  .dims {
    position: absolute; left: 6px; top: 6px; padding: 2px 6px; border-radius: var(--r-1);
    font-size: 11px; font-weight: 600; color: #fff; background: rgb(0 0 0 / .6); pointer-events: none;
  }
  .h {
    position: absolute; width: 12px; height: 12px; margin: -6px 0 0 -6px; border-radius: 2px;
    background: #fff; border: 1px solid rgb(0 0 0 / .35); box-shadow: 0 1px 3px rgb(0 0 0 / .4);
  }
  .h::after { content: ""; position: absolute; inset: -8px; } /* bigger hit target */
  .nw { left: 0; top: 0; cursor: nwse-resize; }
  .n { left: 50%; top: 0; cursor: ns-resize; width: 20px; height: 6px; margin: -3px 0 0 -10px; }
  .ne { left: 100%; top: 0; cursor: nesw-resize; }
  .e { left: 100%; top: 50%; cursor: ew-resize; width: 6px; height: 20px; margin: -10px 0 0 -3px; }
  .se { left: 100%; top: 100%; cursor: nwse-resize; }
  .s { left: 50%; top: 100%; cursor: ns-resize; width: 20px; height: 6px; margin: -3px 0 0 -10px; }
  .sw { left: 0; top: 100%; cursor: nesw-resize; }
  .w { left: 0; top: 50%; cursor: ew-resize; width: 6px; height: 20px; margin: -10px 0 0 -3px; }
</style>
