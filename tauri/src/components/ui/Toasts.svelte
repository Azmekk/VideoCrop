<script lang="ts">
  import { toasts } from "../../lib/state/toast.svelte";
  import Icon from "./Icon.svelte";
</script>

<div class="toasts" aria-live="polite">
  {#each toasts.items as t (t.id)}
    <div class="toast">
      <Icon name="info" size={15} />
      <span>{t.text}</span>
      <button class="btn ghost sm icon" aria-label="Dismiss" onclick={() => toasts.dismiss(t.id)}><Icon name="close" size={13} /></button>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed; left: 50%; bottom: 24px; transform: translateX(-50%); z-index: 900;
    display: flex; flex-direction: column; gap: 8px; align-items: center; pointer-events: none;
  }
  .toast {
    pointer-events: auto; display: flex; align-items: center; gap: 8px;
    padding: 6px 6px 6px 12px; border-radius: var(--r-3);
    background: var(--surface-3); border: 1px solid var(--border-strong); box-shadow: var(--shadow-2);
    color: var(--text); font-size: 12.5px; font-variant-numeric: tabular-nums;
    animation: in var(--med) var(--ease);
  }
  .toast :global(svg:first-child) { color: var(--accent-text); }
  @keyframes in { from { opacity: 0; transform: translateY(6px); } }
</style>
