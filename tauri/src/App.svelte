<script lang="ts">
  import { onMount, type Component } from "svelte";
  import { api } from "./lib/api";
  import { windowControls, onFileDrop } from "./lib/platform";
  import { handleGlobalKey } from "./lib/keyboard";
  import { editor } from "./lib/state/editor.svelte";
  import { settings } from "./lib/state/settings.svelte";
  import { exporter } from "./lib/state/export.svelte";
  import { encoder } from "./lib/state/encode.svelte";
  import { openVideo } from "./lib/state/pipeline.svelte";
  import TitleBar from "./components/TitleBar.svelte";
  import EmptyState from "./components/EmptyState.svelte";
  import ToolsSetup from "./components/ToolsSetup.svelte";
  import Tooltip from "./components/ui/Tooltip.svelte";
  import Toasts from "./components/ui/Toasts.svelte";

  let phase = $state<"boot" | "tools" | "ready">("boot");
  let dragOver = $state(false);

  // The editor (player, timeline, panels) is its own chunk so the first paint stays light.
  // It is preloaded when the browser goes idle, and awaited if a file opens before that.
  let Editor = $state<Component | null>(null);
  let editorP: Promise<void> | null = null;
  const loadEditor = () =>
    (editorP ??= import("./components/Editor.svelte").then((m) => { Editor = m.default; }));
  $effect(() => { if (editor.video && !Editor) void loadEditor(); });

  onMount(() => {
    // Show the (hidden) window after the first paint so it never flashes white.
    requestAnimationFrame(() => requestAnimationFrame(() => windowControls.show()));

    void (async () => {
      const [status] = await Promise.all([
        api.toolsStatus().catch(() => null),
        settings.load(),
        exporter.loadCatalog(),
        encoder.init(),
      ]);
      exporter.restore();
      phase = status && !status.ready ? "tools" : "ready";
      (window.requestIdleCallback ?? ((f: () => void) => setTimeout(f, 300)))(() => void loadEditor());
    })();

    const unDrop = onFileDrop((s) => {
      if (s === "over") dragOver = phase === "ready" && !encoder.busy;
      else if (s === "leave") dragOver = false;
      else if (phase === "ready") void openVideo(s.path);
    });
    return () => { unDrop.then((f) => f()); };
  });
</script>

<svelte:window onkeydown={handleGlobalKey} />

<div class="app">
  <TitleBar />
  <main>
    {#if phase === "tools"}
      <ToolsSetup onready={() => (phase = "ready")} />
    {:else if phase === "ready"}
      {#if editor.video && Editor}
        <Editor />
      {:else}
        <EmptyState {dragOver} />
      {/if}
    {/if}
  </main>
  {#if dragOver && editor.video}
    <div class="drop-veil"><span>Drop to open</span></div>
  {/if}
  <Tooltip />
  <Toasts />
</div>

<style>
  .app { height: 100%; display: flex; flex-direction: column; }
  main { flex: 1; min-height: 0; display: flex; }
  .drop-veil {
    position: fixed; inset: var(--titlebar-h) 0 0 0; z-index: 50; display: grid; place-items: center;
    background: var(--scrim); border: 2px dashed var(--accent); pointer-events: none;
  }
  .drop-veil span {
    padding: 10px 16px; border-radius: var(--r-3); background: var(--surface); font-weight: 600;
    box-shadow: var(--shadow-2);
  }
</style>
