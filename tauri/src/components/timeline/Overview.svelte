<script lang="ts">
  // Mini-map of the whole file: cut range, playhead, and the visible window (draggable).
  import { timeline } from "../../lib/state/timeline.svelte";
  import { cut } from "../../lib/state/cut.svelte";
  import { editor } from "../../lib/state/editor.svelte";

  let width = $state(0);
  let drag: { x: number; start: number } | null = null;

  const dur = $derived(timeline.duration);
  const pc = (t: number) => (t / dur) * 100;

  function down(e: PointerEvent) {
    if (e.button !== 0) return;
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const t = ((e.clientX - r.left) / r.width) * dur;
    const v = timeline.view;
    // Clicking outside the window centers it there; then drag continues from the new position.
    if (t < v.start || t > v.start + v.span) timeline.set({ start: t - v.span / 2, span: v.span });
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    drag = { x: e.clientX, start: timeline.view.start };
  }

  function move(e: PointerEvent) {
    if (!drag || width <= 0) return;
    const dt = ((e.clientX - drag.x) / width) * dur;
    timeline.set({ start: drag.start + dt, span: timeline.view.span });
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="ov" class:zoomed={timeline.zoomed} bind:clientWidth={width}
  onpointerdown={down} onpointermove={move} onpointerup={() => (drag = null)} onpointercancel={() => (drag = null)}
  data-tip={timeline.zoomed ? "Drag to move the visible part of the timeline" : undefined}
>
  <div class="rail"></div>
  <div class="cut" style:left="{pc(cut.start)}%" style:width="{pc(cut.end - cut.start)}%"></div>
  <div class="ph" style:left="{pc(editor.displayTime)}%"></div>
  <div class="win" style:left="{pc(timeline.view.start)}%" style:width="{pc(timeline.view.span)}%"></div>
</div>

<style>
  .ov { position: relative; height: 12px; margin-top: 4px; touch-action: none; }
  .rail { position: absolute; left: 0; right: 0; top: 5px; height: 2px; border-radius: 2px; background: var(--surface-3); }
  .cut { position: absolute; top: 5px; height: 2px; background: var(--accent); border-radius: 2px; }
  .ph { position: absolute; top: 2px; height: 8px; width: 2px; margin-left: -1px; background: var(--text); border-radius: 1px; }
  .win {
    position: absolute; top: 0; bottom: 0; min-width: 6px; border-radius: 3px;
    border: 1px solid var(--border-strong); background: rgb(127 127 127 / .08);
    opacity: 0; transition: opacity var(--fast);
  }
  .zoomed .win { opacity: 1; cursor: grab; }
  .zoomed .win:hover { border-color: var(--text-3); }
</style>
