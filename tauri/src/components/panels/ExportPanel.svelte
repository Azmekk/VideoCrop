<script lang="ts">
  import { exporter, type ExportTab } from "../../lib/state/export.svelte";
  import { editor } from "../../lib/state/editor.svelte";
  import { pickFolder } from "../../lib/platform";
  import { dirname } from "../../lib/util/format";
  import type { OutputMode } from "../../lib/types";
  import Section from "./Section.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import Switch from "../ui/Switch.svelte";
  import Icon from "../ui/Icon.svelte";
  import ShareTab from "../export/ShareTab.svelte";
  import QualityTab from "../export/QualityTab.svelte";
  import CustomTab from "../export/CustomTab.svelte";

  const tabs: { value: ExportTab; label: string }[] = [
    { value: "share", label: "Share" }, { value: "quality", label: "Quality" }, { value: "custom", label: "Custom" },
  ];
  const modes: { value: OutputMode; label: string; tip: string }[] = [
    { value: "everything", label: "Everything", tip: "Video and audio" },
    { value: "audioOnly", label: "Audio only", tip: "Extract the soundtrack" },
    { value: "videoOnly", label: "Video only", tip: "Drop the audio" },
  ];

  const status = $derived.by(() => {
    if (!exporter.reencode) return "Stream copy";
    const c = exporter.catalog;
    if (exporter.tab === "share") return c?.platforms.find((p) => p.id === exporter.platformId)?.name ?? "Pick a destination";
    if (exporter.tab === "quality") return c?.quality.find((q) => q.id === exporter.qualityId)?.name ?? "";
    return `Custom ${exporter.custom.codec.toUpperCase()}`;
  });

  function setTab(t: ExportTab) {
    if (t === "custom" && exporter.tab !== "custom") exporter.seedCustomFromSelection();
    exporter.tab = t;
    exporter.persist();
  }

  async function chooseFolder() {
    const d = await pickFolder(exporter.outputDir ?? (editor.info ? dirname(editor.info.path) : null));
    if (d) { exporter.outputDir = d; exporter.persist(); }
  }
</script>

<Section title="Export" icon="export" {status} active>
  {#snippet actions()}
    <span class="re label">Re-encode</span>
    <Switch checked={exporter.reencode} onchange={(v) => { exporter.reencode = v; exporter.persist(); }} label="Re-encode" />
  {/snippet}

  {#if exporter.reencode}
    <Segmented label="Compression" value={exporter.tab} options={tabs} onchange={setTab} />
    {#if exporter.tab === "share"}<ShareTab />
    {:else if exporter.tab === "quality"}<QualityTab />
    {:else}<CustomTab />{/if}
  {:else}
    <p class="note copy">
      <strong>Stream copy.</strong> The original video and audio are copied as-is: very fast and no quality loss.
      Crop and resize are skipped, and cuts snap to keyframes.
    </p>
  {/if}

  <div class="field-row">
    <span class="label">Output</span>
    <Segmented size="sm" label="Output streams" value={exporter.mode} options={modes} onchange={(m) => (exporter.mode = m)} />
  </div>

  <div class="field-row">
    <span class="label">Save to</span>
    <div class="folder">
      <span class="path" title={exporter.outputDir ?? ""}>
        <Icon name="folder" size={14} />
        <span class="txt">{exporter.outputDir ?? "Next to the source file"}</span>
      </span>
      <button class="btn sm" onclick={chooseFolder}>Change…</button>
      {#if exporter.outputDir}
        <button class="btn ghost sm icon" onclick={() => { exporter.outputDir = null; exporter.persist(); }} aria-label="Save next to the source" data-tip="Save next to the source file"><Icon name="close" size={12} /></button>
      {/if}
    </div>
  </div>
</Section>

<style>
  .re { margin-right: 6px; }
  .copy strong { color: var(--text); font-weight: 600; }
  .field-row { display: flex; flex-direction: column; gap: 5px; margin-top: 2px; }
  .folder { display: flex; align-items: center; gap: 4px; min-width: 0; }
  .path {
    flex: 1; min-width: 0; display: flex; align-items: center; gap: 7px; height: 28px; padding: 0 8px;
    border-radius: var(--r-2); background: var(--surface-2); color: var(--text-2); font-size: 12.5px;
  }
  .path :global(svg) { flex: none; color: var(--text-3); }
  .txt { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; direction: rtl; text-align: left; }
</style>
