<script lang="ts">
  import { editor } from "../lib/state/editor.svelte";
  import { encoder } from "../lib/state/encode.svelte";
  import { buildDraft, schedulePlan } from "../lib/state/pipeline.svelte";
  import Stage from "./stage/Stage.svelte";
  import Transport from "./stage/Transport.svelte";
  import CropToolbar from "./stage/CropToolbar.svelte";
  import Timeline from "./timeline/Timeline.svelte";
  import CutPanel from "./panels/CutPanel.svelte";
  import CropPanel from "./panels/CropPanel.svelte";
  import ResizePanel from "./panels/ResizePanel.svelte";
  import ExportPanel from "./panels/ExportPanel.svelte";
  import SummaryPanel from "./panels/SummaryPanel.svelte";
  import ProcessBar from "./ProcessBar.svelte";
  import SizeConfirm from "./SizeConfirm.svelte";

  // Re-plan on every edit; the backend owns the summary, output name and estimate.
  $effect(() => {
    const draft = buildDraft();
    JSON.stringify(draft); // track every nested field
    if (!encoder.busy) schedulePlan(draft);
  });
</script>

<div class="editor">
  <div class="left" inert={encoder.busy}>
    <Stage />
    {#if editor.mode === "crop"}
      <CropToolbar />
    {:else}
      <Transport />
    {/if}
    <div class="tl" inert={editor.mode === "crop"} class:off={editor.mode === "crop"}>
      <Timeline />
    </div>
  </div>

  <aside class="panel">
    <div class="scroll">
      <fieldset disabled={encoder.busy}>
        <legend class="sr-only">Edit settings</legend>
        <CutPanel />
        <CropPanel />
        <ResizePanel />
        <ExportPanel />
        <SummaryPanel />
      </fieldset>
    </div>
    <ProcessBar />
  </aside>
</div>
<SizeConfirm />

<style>
  .editor { flex: 1; min-width: 0; display: flex; }
  .left { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .tl.off { opacity: .45; transition: opacity var(--med); }
  .panel {
    flex: none; width: var(--panel-w); display: flex; flex-direction: column; min-height: 0;
    background: var(--panel); border-left: 1px solid var(--border);
  }
  .scroll { flex: 1; min-height: 0; overflow-y: auto; overflow-x: hidden; scrollbar-gutter: stable; }
  fieldset { border: 0; margin: 0; padding: 0; min-width: 0; }
  fieldset:disabled { opacity: .55; }
</style>
