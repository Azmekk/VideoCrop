<script lang="ts">
  import { crop, ASPECTS } from "../../lib/state/crop.svelte";
  import { exitCropMode } from "../../lib/state/pipeline.svelte";
  import Icon from "../ui/Icon.svelte";
</script>

<div class="bar">
  <div class="chips" role="group" aria-label="Aspect ratio">
    {#each ASPECTS as a (a.id)}
      <button class="chip" aria-pressed={crop.aspect === a.id} onclick={() => crop.chooseAspect(a.id)}>{a.label}</button>
    {/each}
  </div>
  <div class="spacer"></div>
  <button class="btn ghost icon" onclick={() => crop.resetDraft()} aria-label="Reset crop" data-tip="Back to the full frame"><Icon name="reset" size={14} /></button>
  <button class="btn" onclick={() => exitCropMode(false)}>Cancel</button>
  <button class="btn primary" onclick={() => exitCropMode(true)}><Icon name="check" size={14} /> Apply</button>
</div>

<style>
  .bar {
    flex: none; display: flex; align-items: center; gap: 8px; padding: 8px 12px;
    background: var(--panel); border-top: 1px solid var(--border);
  }
  .chips { flex-wrap: nowrap; overflow-x: auto; min-width: 0; scrollbar-width: none; }
  .chip { flex: none; }
  .spacer { flex: 1; }
</style>
