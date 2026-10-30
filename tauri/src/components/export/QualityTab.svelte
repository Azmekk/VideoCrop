<script lang="ts">
  import { exporter } from "../../lib/state/export.svelte";
  import Icon from "../ui/Icon.svelte";

  const groups = $derived.by(() => {
    const q = exporter.catalog?.quality ?? [];
    return [
      { title: "Compatibility", items: q.filter((p) => p.category === "compatibility") },
      { title: "Optimization", items: q.filter((p) => p.category === "optimization") },
    ].filter((g) => g.items.length);
  });
</script>

{#each groups as g (g.title)}
  <div class="group" role="radiogroup" aria-label={g.title}>
    <h3 class="label">{g.title}</h3>
    {#each g.items as p (p.id)}
      {@const on = exporter.qualityId === p.id}
      <button class="opt" role="radio" aria-checked={on} onclick={() => exporter.selectQuality(p.id)}>
        <span class="dot" aria-hidden="true"></span>
        <span class="txt">
          <span class="name">{p.name}</span>
          <span class="desc">{p.description}</span>
          {#if on && p.warning}
            <span class="warn"><Icon name="alert" size={13} /><span>{p.warning}</span></span>
          {/if}
        </span>
      </button>
    {/each}
  </div>
{/each}

<style>
  .group { display: flex; flex-direction: column; gap: 2px; }
  h3 { margin: 2px 0 4px; font-weight: 500; }
  .opt {
    display: flex; gap: 10px; align-items: flex-start; padding: 8px 9px; text-align: left;
    border: 1px solid transparent; border-radius: var(--r-3); background: transparent;
    transition: background var(--fast), border-color var(--fast);
  }
  .opt:hover { background: var(--surface-2); }
  .opt[aria-checked="true"] { background: var(--surface); border-color: var(--border-strong); }
  .dot {
    flex: none; width: 14px; height: 14px; margin-top: 2px; border-radius: 50%;
    border: 1.5px solid var(--border-strong); transition: border var(--fast);
  }
  .opt[aria-checked="true"] .dot { border: 4px solid var(--accent); }
  .txt { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .name { font-weight: 600; font-size: 12.5px; }
  .desc { font-size: 12px; color: var(--text-2); line-height: 1.4; }
  .warn { margin-top: 5px; }
</style>
