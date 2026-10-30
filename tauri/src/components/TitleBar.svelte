<script lang="ts">
  import { onMount } from "svelte";
  import { inTauri } from "../lib/api";
  import { windowControls } from "../lib/platform";
  import { settings } from "../lib/state/settings.svelte";
  import { editor } from "../lib/state/editor.svelte";
  import { encoder } from "../lib/state/encode.svelte";
  import { chooseAndOpen, closeVideo } from "../lib/state/pipeline.svelte";
  import { basename } from "../lib/util/format";
  import Icon from "./ui/Icon.svelte";
  import Segmented from "./ui/Segmented.svelte";

  let maximized = $state(false);
  onMount(() => {
    const un = windowControls.onResized((m) => (maximized = m));
    return () => { un.then((f) => f()); };
  });

  const themes = [
    { value: "system" as const, icon: "system", tip: "Match Windows" },
    { value: "light" as const, icon: "sun", tip: "Light" },
    { value: "dark" as const, icon: "moon", tip: "Dark" },
  ];
</script>

<header class="bar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <!-- Small-size rendition of the app icon (assets/icon-proposal.svg). -->
    <svg class="mark" width="18" height="18" viewBox="0 0 18 18" aria-hidden="true">
      <defs>
        <linearGradient id="vc-tile" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stop-color="#38BDF8" /><stop offset="1" stop-color="#1D4ED8" />
        </linearGradient>
      </defs>
      <rect x="0.5" y="0.5" width="17" height="17" rx="4" fill="url(#vc-tile)" />
      <rect x="3.5" y="5.5" width="11" height="7" rx="1" fill="none" stroke="#fff" stroke-width="1.1" />
      <path d="M5.5 9V7.5H7M11 7.5h1.5V9M12.5 9.5V10.5H11M7 10.5H5.5V9.5" fill="none" stroke="#fff" stroke-width="1.1" stroke-linecap="round" />
    </svg>
    <span class="name" data-tauri-drag-region>VideoCrop</span>
  </div>

  {#if editor.video}
    <div class="file" data-tauri-drag-region>
      <span class="filename" data-tauri-drag-region title={editor.video.info.path}>{basename(editor.video.info.path)}</span>
      <button class="btn ghost sm" disabled={encoder.busy} onclick={chooseAndOpen} data-tip="Open another video (Ctrl+O)">
        <Icon name="folder" size={14} /> Open
      </button>
      <button class="btn ghost sm icon" disabled={encoder.busy} onclick={closeVideo} aria-label="Close video" data-tip="Close video">
        <Icon name="close" size={13} />
      </button>
    </div>
  {/if}

  <div class="spacer" data-tauri-drag-region></div>

  <div class="theme">
    <Segmented size="sm" label="Theme" value={settings.value.theme} options={themes} onchange={(t) => settings.setTheme(t)} />
  </div>

  {#if inTauri}
    <div class="wc">
      <button aria-label="Minimize" onclick={() => windowControls.minimize()}><Icon name="min" size={14} /></button>
      <button aria-label={maximized ? "Restore" : "Maximize"} onclick={() => windowControls.toggleMaximize()}>
        <Icon name={maximized ? "restore" : "max"} size={13} />
      </button>
      <button class="close" aria-label="Close" onclick={() => windowControls.close()}><Icon name="close" size={14} /></button>
    </div>
  {:else}
    <div class="wc-pad"></div>
  {/if}
</header>

<style>
  .bar {
    flex: none; height: var(--titlebar-h); display: flex; align-items: center; gap: 12px;
    padding-left: 12px; background: var(--bg); border-bottom: 1px solid var(--border);
  }
  .brand { display: flex; align-items: center; gap: 8px; color: var(--text); }
  .mark { color: var(--text-2); }
  .name { font: 600 13px/1 var(--font-display); letter-spacing: .01em; }
  .file { display: flex; align-items: center; gap: 2px; min-width: 0; padding-left: 12px; border-left: 1px solid var(--border); }
  .filename { color: var(--text-2); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 36vw; margin-right: 6px; }
  .spacer { flex: 1; align-self: stretch; }
  .theme { margin-right: 4px; }
  .wc { display: flex; align-self: stretch; }
  .wc button {
    width: 46px; border: 0; background: transparent; color: var(--text-2);
    display: grid; place-items: center; transition: background var(--fast), color var(--fast);
  }
  .wc button:hover { background: var(--surface-2); color: var(--text); }
  .wc .close:hover { background: #c42b1c; color: #fff; }
  .wc-pad { width: 4px; }
</style>
