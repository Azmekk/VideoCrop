<script lang="ts">
  import { formatTick } from "../../lib/util/time";
  import { minorStep, tickStep, ticks, timeToX, type View } from "../../lib/util/zoom";

  let { view, width, fps }: { view: View; width: number; fps: number } = $props();

  const step = $derived(tickStep(view.span, width, 84, fps));
  const major = $derived(ticks(view, step));
  const mStep = $derived(minorStep(step, fps));
  const minor = $derived(ticks(view, mStep).filter((t) => Math.abs(t / step - Math.round(t / step)) > 1e-4));
</script>

<div class="ruler" aria-hidden="true">
  {#each minor as t (t)}
    <span class="mi" style:left="{timeToX(t, view, width)}px"></span>
  {/each}
  {#each major as t (t)}
    <span class="ma" style:left="{timeToX(t, view, width)}px"><b class="tnum">{formatTick(t, step)}</b></span>
  {/each}
</div>

<style>
  .ruler { position: relative; height: 20px; overflow: hidden; }
  .mi, .ma { position: absolute; bottom: 0; width: 1px; background: var(--border-strong); }
  .mi { height: 4px; }
  .ma { height: 8px; background: var(--text-3); }
  b {
    position: absolute; left: 4px; bottom: 3px; font-size: 10.5px; font-weight: 500;
    color: var(--text-3); white-space: nowrap; font-variant-numeric: tabular-nums;
  }
</style>
