<script lang="ts">
  import { process, sizeConfirm } from "../lib/state/pipeline.svelte";
  import Icon from "./ui/Icon.svelte";

  let back = $state<HTMLButtonElement>();

  // The safe choice gets focus, so Enter doesn't start a doomed encode by accident.
  $effect(() => { if (sizeConfirm.open) back?.focus(); });

  function onkey(e: KeyboardEvent) {
    if (!sizeConfirm.open) return;
    if (e.key === "Escape") { e.preventDefault(); e.stopPropagation(); sizeConfirm.open = false; }
  }
</script>

<svelte:window onkeydowncapture={onkey} />

{#if sizeConfirm.open}
  <div class="scrim" role="presentation" onclick={() => (sizeConfirm.open = false)}></div>
  <div class="dialog" role="alertdialog" aria-modal="true" aria-labelledby="sc-title" aria-describedby="sc-body">
    <div class="head">
      <span class="badge"><Icon name="alert" size={16} /></span>
      <h2 id="sc-title">This is going to look rough</h2>
    </div>
    <p id="sc-body">{sizeConfirm.message}</p>
    <div class="actions">
      <button class="btn" onclick={() => process(true)}>Encode anyway</button>
      <button class="btn primary" bind:this={back} onclick={() => (sizeConfirm.open = false)}>Go back</button>
    </div>
  </div>
{/if}

<style>
  .scrim { position: fixed; inset: 0; z-index: 960; background: var(--scrim); animation: fade var(--med) var(--ease); }
  .dialog {
    position: fixed; left: 50%; top: 50%; z-index: 961; width: min(440px, calc(100vw - 32px));
    transform: translate(-50%, -50%); display: flex; flex-direction: column; gap: 12px; padding: 18px;
    border-radius: var(--r-4); background: var(--surface); border: 1px solid var(--border-strong);
    box-shadow: var(--shadow-2); animation: pop var(--med) var(--ease);
  }
  .head { display: flex; align-items: center; gap: 10px; }
  .badge {
    display: grid; place-items: center; width: 30px; height: 30px; border-radius: var(--r-2);
    background: var(--danger-soft); color: var(--danger); flex: none;
  }
  h2 { margin: 0; font: 600 15px/1.3 var(--font-display); }
  p { margin: 0; font-size: 13px; line-height: 1.55; color: var(--text-2); user-select: text; }
  .actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
  @keyframes fade { from { opacity: 0; } }
  @keyframes pop { from { opacity: 0; transform: translate(-50%, calc(-50% + 6px)) scale(.98); } }
  @media (prefers-reduced-motion: reduce) { .scrim, .dialog { animation: none; } }
</style>
