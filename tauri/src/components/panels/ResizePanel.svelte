<script lang="ts">
  import { editor } from "../../lib/state/editor.svelte";
  import { crop } from "../../lib/state/crop.svelte";
  import { resize, HEIGHT_PRESETS, PERCENT_PRESETS } from "../../lib/state/resize.svelte";
  import { aspectLabel, evenRound } from "../../lib/util/geometry";
  import Section from "./Section.svelte";
  import Switch from "../ui/Switch.svelte";
  import Icon from "../ui/Icon.svelte";

  const info = $derived(editor.info!);
  const inW = $derived(resize.inW);
  const inH = $derived(resize.inH);
  const status = $derived(resize.enabled ? `${resize.width}×${resize.height}` : "Original size");
  const presets = $derived.by(() => {
    const list = HEIGHT_PRESETS.map((h) => ({ h, original: h === evenRound(inH) }));
    if (!list.some((p) => p.original)) list.push({ h: evenRound(inH), original: true });
    return list.sort((a, b) => b.h - a.h);
  });
  const pctActive = (p: number) =>
    resize.enabled && resize.width === Math.max(2, evenRound((inW * p) / 100)) && resize.height === Math.max(2, evenRound((inH * p) / 100));

  function num(e: Event & { currentTarget: HTMLInputElement }, set: (n: number) => void, get: () => number) {
    const n = Number(e.currentTarget.value);
    if (Number.isFinite(n) && n >= 2) set(n);
    e.currentTarget.value = String(get());
  }
</script>

<Section title="Resize" icon="resize" {status} active={resize.enabled && !!resize.toSpec()}>
  {#snippet actions()}
    <Switch checked={resize.enabled} onchange={(v) => { resize.enabled = v; if (v && !resize.width) resize.reset(); }} label="Resize output" />
  {/snippet}

  <p class="input tnum">
    <span class="faint">Input</span> {inW}×{inH} <span class="faint">({aspectLabel(crop.displayDims.width, crop.displayDims.height)})</span>
    {#if crop.spec}<span class="faint">— cropped from {info.width}×{info.height}</span>{/if}
  </p>

  {#if resize.enabled}
    <div class="dims">
      <label class="num"><span class="label">Width</span>
        <input class="field" type="number" min="2" step="2" value={resize.width} onchange={(e) => num(e, (n) => resize.setWidth(n), () => resize.width)} />
      </label>
      <button class="btn icon lock" class:on={resize.locked} aria-pressed={resize.locked} onclick={() => resize.setLocked(!resize.locked)}
        aria-label="Lock aspect ratio" data-tip={resize.locked ? "Aspect locked to the input. Click to unlock." : "Aspect unlocked. Click to lock to the input."}>
        <Icon name={resize.locked ? "lock" : "unlock"} size={14} />
      </button>
      <label class="num"><span class="label">Height</span>
        <input class="field" type="number" min="2" step="2" value={resize.height} onchange={(e) => num(e, (n) => resize.setHeight(n), () => resize.height)} />
      </label>
    </div>

    {#if resize.mismatch}
      <div class="warn"><Icon name="alert" size={14} /><span>Aspect mismatch — output will be stretched.</span></div>
    {/if}
  {/if}

  <div class="chips" role="group" aria-label="Height presets">
    {#each presets as p (p.h)}
      {@const up = p.h > inH}
      <span data-tip={up ? "Would upscale — larger than the input" : undefined}>
        <button class="chip" disabled={up} aria-pressed={resize.enabled && resize.height === p.h && !resize.mismatch}
          onclick={() => resize.presetHeight(p.h)}>{p.h}p{#if p.original}<em>original</em>{/if}</button>
      </span>
    {/each}
  </div>
  <div class="chips" role="group" aria-label="Scale">
    {#each PERCENT_PRESETS as p (p)}
      <button class="chip" aria-pressed={pctActive(p)} onclick={() => resize.percent(p)}>{p}%</button>
    {/each}
  </div>
</Section>

<style>
  .input { margin: 0; font-size: 12.5px; }
  .dims { display: grid; grid-template-columns: 1fr auto 1fr; gap: 6px; align-items: end; }
  .num { display: flex; flex-direction: column; gap: 3px; }
  .num input { width: 100%; }
  .chip em { font-style: normal; margin-left: 5px; opacity: .7; font-weight: 400; }
  .lock { color: var(--text-3); }
  .lock.on { color: var(--accent-text); border-color: var(--accent-line); background: var(--accent-soft); }
</style>
