<script lang="ts">
  import { editor } from "../../lib/state/editor.svelte";
  import { crop } from "../../lib/state/crop.svelte";
  import { applyCrop, enterCropMode, exitCropMode } from "../../lib/state/pipeline.svelte";
  import { aspectLabel, normalizeCrop } from "../../lib/util/geometry";
  import type { CropSpec } from "../../lib/types";
  import Section from "./Section.svelte";
  import Switch from "../ui/Switch.svelte";
  import Icon from "../ui/Icon.svelte";

  const editing = $derived(editor.mode === "crop");
  const info = $derived(editor.info!);
  const shown = $derived(editing ? normalizeCrop(crop.draft, info.width, info.height) : crop.spec);
  const status = $derived(
    shown && (editing || crop.spec)
      ? `${shown.width}×${shown.height} (${aspectLabel(crop.displayDimsFor(shown).width, crop.displayDimsFor(shown).height)})`
      : "Full frame",
  );

  const FIELDS: { k: keyof CropSpec; label: string }[] = [
    { k: "x", label: "X" }, { k: "y", label: "Y" }, { k: "width", label: "W" }, { k: "height", label: "H" },
  ];

  function setField(k: keyof CropSpec, raw: string) {
    const n = Number(raw);
    if (!Number.isFinite(n)) return;
    const next = { ...crop.normalizedDraft(), [k]: n };
    const r = crop.ratioFor(crop.aspect);
    if (r && k === "width") next.height = next.width / r;
    if (r && k === "height") next.width = next.height * r;
    crop.setDraft(normalizeCrop(next, info.width, info.height));
  }
</script>

<Section title="Crop" icon="crop" {status} active={!!crop.spec || editing}>
  {#snippet actions()}
    {#if editing}
      <button class="btn ghost sm" onclick={() => exitCropMode(false)} data-tip="Discard changes (Esc)">Cancel</button>
      <button class="btn primary sm" onclick={() => exitCropMode(true)} data-tip="Apply crop (Enter)"><Icon name="check" size={13} /> Apply</button>
    {:else if crop.spec}
      <button class="btn ghost sm" onclick={() => applyCrop(null)} data-tip="Remove the crop"><Icon name="reset" size={13} /> Reset</button>
    {/if}
  {/snippet}

  {#if editing}
    <div class="xywh">
      {#each FIELDS as f (f.k)}
        <label class="num">
          <span class="label">{f.label}</span>
          <input class="field" type="number" step="2" min="0"
            value={shown?.[f.k] ?? 0}
            onchange={(e) => setField(f.k, e.currentTarget.value)} />
        </label>
      {/each}
    </div>
    <p class="note">Drag on the video to draw, or use the handles. Arrow keys nudge, Ctrl+arrows resize. Enter applies, Esc cancels. Values are source pixels, rounded to even numbers.</p>
  {:else}
    <div class="row">
      <button class="btn" onclick={enterCropMode}><Icon name="crop" size={14} /> {crop.spec ? "Edit crop" : "Crop video"}</button>
      {#if crop.spec}
        <label class="toggle">
          <Switch checked={editor.previewCropped} onchange={(v) => (editor.previewCropped = v)} label="Preview cropped" />
          <span>Preview cropped</span>
        </label>
      {/if}
    </div>
    {#if crop.spec}
      <p class="note tnum">Offset {crop.spec.x}, {crop.spec.y} in the {info.width}×{info.height} source.</p>
    {/if}
  {/if}
</Section>

<style>
  .xywh { display: grid; grid-template-columns: repeat(4, 1fr); gap: 6px; }
  .num { display: flex; flex-direction: column; gap: 3px; }
  .num input { width: 100%; padding-right: 2px; }
  .row { display: flex; align-items: center; gap: 8px; }
  .toggle { display: flex; align-items: center; gap: 8px; margin-left: auto; font-size: 12.5px; color: var(--text-2); }
</style>
