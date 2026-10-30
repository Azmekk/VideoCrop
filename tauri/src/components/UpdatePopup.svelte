<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/api";
  import { settings } from "../lib/state/settings.svelte";
  import type { UpdateInfo } from "../lib/types";
  import Icon from "./ui/Icon.svelte";

  // Checked once per launch, a few seconds in, so it never competes with startup.
  const CHECK_DELAY_MS = 4000;

  let update = $state<UpdateInfo | null>(null);

  onMount(() => {
    const t = setTimeout(async () => {
      try {
        const u = await api.checkUpdate();
        if (u && u.version !== settings.value.skippedUpdate) update = u;
      } catch (e) {
        console.info("update check skipped:", e);
      }
    }, CHECK_DELAY_MS);
    return () => clearTimeout(t);
  });

  function download() {
    if (!update) return;
    void api.openReleasePage(update.url);
    update = null;
  }

  function skip() {
    if (!update) return;
    settings.update({ skippedUpdate: update.version });
    update = null;
  }
</script>

{#if update}
  <div class="update" role="dialog" aria-labelledby="update-title">
    <div class="head">
      <span class="badge"><Icon name="download" size={15} /></span>
      <div>
        <div id="update-title" class="title">VideoCrop {update.version} is available</div>
        <div class="sub">Download the new version from the website when you're ready.</div>
      </div>
      <button class="btn ghost sm icon" aria-label="Later" data-tip="Remind me next launch" onclick={() => (update = null)}>
        <Icon name="close" size={13} />
      </button>
    </div>
    {#if update.notes}
      <div class="notes">{update.notes}</div>
    {/if}
    <div class="actions">
      <button class="btn ghost sm" onclick={skip}>Skip this version</button>
      <button class="btn primary sm" onclick={download}>Download</button>
    </div>
  </div>
{/if}

<style>
  .update {
    position: fixed; right: 16px; bottom: 16px; z-index: 950; width: 340px;
    display: flex; flex-direction: column; gap: 10px; padding: 12px;
    border-radius: var(--r-3); background: var(--surface-3); border: 1px solid var(--border-strong);
    box-shadow: var(--shadow-2); animation: in var(--med) var(--ease);
  }
  .head { display: grid; grid-template-columns: auto 1fr auto; gap: 10px; align-items: start; }
  .badge {
    display: grid; place-items: center; width: 28px; height: 28px; border-radius: var(--r-2);
    background: var(--accent); color: var(--accent-ink);
  }
  .title { font-weight: 600; font-size: 13px; }
  .sub { color: var(--text-2); font-size: 12px; margin-top: 2px; }
  .notes {
    max-height: 120px; overflow: auto; white-space: pre-wrap; font-size: 12px; line-height: 1.5;
    color: var(--text-2); padding: 8px 10px; border-radius: var(--r-2); background: var(--surface-2);
  }
  .actions { display: flex; justify-content: flex-end; gap: 6px; }
  @keyframes in { from { opacity: 0; transform: translateY(8px); } }
</style>
