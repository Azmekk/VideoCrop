<script lang="ts">
  import { exporter } from "../../lib/state/export.svelte";
  import Icon from "../ui/Icon.svelte";

  const lines = $derived((exporter.plan?.summary ?? "").split("\n").filter(Boolean));
</script>

<section class="sum" aria-live="polite">
  <h2>What will happen</h2>
  {#if exporter.planError}
    <div class="err" role="alert"><Icon name="alert" size={14} /><span>{exporter.planError}</span></div>
  {:else if lines.length}
    <ol>
      {#each lines as l, i (i)}<li>{l}</li>{/each}
    </ol>
  {:else}
    <p class="note">Working out the plan…</p>
  {/if}
  {#each exporter.plan?.warnings ?? [] as w (w)}
    <div class="warn"><Icon name="alert" size={14} /><span>{w}</span></div>
  {/each}
</section>

<style>
  .sum { padding: 14px 16px 18px; display: flex; flex-direction: column; gap: 8px; }
  h2 { margin: 0 0 2px; font: 600 13.5px/1 var(--font-display); }
  ol { margin: 0; padding: 0; list-style: none; display: flex; flex-direction: column; gap: 0; counter-reset: s; }
  li {
    position: relative; padding: 5px 0 5px 22px; font-size: 12.5px; line-height: 1.4; color: var(--text);
    font-variant-numeric: tabular-nums; user-select: text; counter-increment: s;
  }
  li::before {
    content: counter(s); position: absolute; left: 0; top: 6px; width: 15px; height: 15px; border-radius: 50%;
    display: grid; place-items: center; font-size: 10px; font-weight: 600; color: var(--text-2); background: var(--surface-2);
  }
  li + li::after {
    content: ""; position: absolute; left: 7px; top: -4px; height: 8px; width: 1px; background: var(--border-strong);
  }
  .err { display: flex; gap: 8px; padding: 8px 10px; border-radius: var(--r-2); background: var(--danger-soft); font-size: 12px; user-select: text; }
  .err :global(svg) { color: var(--danger); flex: none; margin-top: 1px; }
</style>
