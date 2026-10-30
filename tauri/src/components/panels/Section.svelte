<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "../ui/Icon.svelte";

  let { title, icon, status, active = false, children, actions }: {
    title: string; icon: string; status?: string; active?: boolean; children: Snippet; actions?: Snippet;
  } = $props();
</script>

<section class="sec" class:active>
  <header>
    <span class="ic"><Icon name={icon} size={15} /></span>
    <h2>{title}</h2>
    {#if status}<span class="status tnum">{status}</span>{/if}
    {#if actions}<div class="actions">{@render actions()}</div>{/if}
  </header>
  <div class="body">{@render children()}</div>
</section>

<style>
  .sec { padding: 14px 16px 16px; border-bottom: 1px solid var(--border); }
  header { display: flex; align-items: center; gap: 8px; min-height: 24px; margin-bottom: 10px; }
  .ic { color: var(--text-3); display: grid; transition: color var(--fast); }
  .active .ic { color: var(--accent-text); }
  h2 { margin: 0; font: 600 13.5px/1 var(--font-display); }
  .status { font-size: 12px; color: var(--text-2); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; }
  .active .status { color: var(--accent-text); }
  .actions { margin-left: auto; display: flex; gap: 2px; }
  .body { display: flex; flex-direction: column; gap: 10px; }
</style>
