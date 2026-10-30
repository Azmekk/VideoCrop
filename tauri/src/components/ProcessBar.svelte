<script lang="ts">
  import { api } from "../lib/api";
  import { exporter } from "../lib/state/export.svelte";
  import { encoder } from "../lib/state/encode.svelte";
  import { editor } from "../lib/state/editor.svelte";
  import { chooseAndOpen, process } from "../lib/state/pipeline.svelte";
  import { basename, formatBytes, formatEta } from "../lib/util/format";
  import Icon from "./ui/Icon.svelte";

  const plan = $derived(exporter.plan);
  const st = $derived(encoder.status);
  const pct = $derived(Math.round((st?.fraction ?? 0) * 100));
  const outName = $derived(encoder.result?.outputPath ?? encoder.plan?.outputPath ?? plan?.outputPath ?? "");
  let showLog = $state(false);

  function openAnother() {
    encoder.dismiss();
    void chooseAndOpen();
  }
</script>

<div class="pb" class:running={encoder.busy}>
  {#if encoder.phase === "idle"}
    <div class="out">
      <div class="name" title={plan?.outputPath}>{plan ? basename(plan.outputPath) : "—"}</div>
      <div class="meta tnum">
        {#if plan?.outputWidth && plan?.outputHeight}<span>{plan.outputWidth}×{plan.outputHeight}</span>{/if}
        {#if plan?.estimatedSizeBytes}<span>about {formatBytes(plan.estimatedSizeBytes)}</span>{/if}
        {#if plan?.twoPass}<span>two-pass</span>{/if}
      </div>
    </div>
    <button class="go" onclick={process} disabled={!plan || !!exporter.planError || editor.mode === "crop" || !exporter.ready}
      data-tip={editor.mode === "crop" ? "Apply or cancel the crop first" : !exporter.ready ? "Pick a destination first" : "Ctrl+Enter"}>
      Process
    </button>
  {:else if encoder.busy}
    <div class="prog">
      <div class="top tnum">
        <span class="big">{pct}%</span>
        <span class="muted">{encoder.phase === "starting" ? "Starting…" : st && st.passCount > 1 ? `Pass ${st.pass}/${st.passCount}` : "Encoding"}</span>
        <span class="spacer"></span>
        <span class="faint">{formatEta(st?.etaSec ?? null)} left</span>
      </div>
      <div class="bar" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={pct}><div style:width="{pct}%"></div></div>
      <div class="stats tnum faint">
        <span>{st ? st.fps.toFixed(0) : "—"} fps</span><span>{st ? st.speed.toFixed(2) : "—"}× realtime</span>
        <span class="file" title={outName}>{basename(outName)}</span>
      </div>
    </div>
    <button class="btn cancel" onclick={() => encoder.cancel()}>Cancel</button>
  {:else if encoder.phase === "done"}
    <div class="result ok">
      <span class="badge"><Icon name="check" size={15} /></span>
      <div class="rt">
        <div class="name" title={outName}>{basename(outName)}</div>
        <div class="meta tnum">Saved, {formatBytes(encoder.result?.outputSizeBytes)}</div>
      </div>
      <button class="btn ghost sm icon" onclick={() => encoder.dismiss()} aria-label="Back to editing" data-tip="Back to editing"><Icon name="close" size={13} /></button>
    </div>
    <div class="actions">
      <button class="btn" onclick={() => api.revealInFolder(outName)}><Icon name="reveal" size={14} /> Show in folder</button>
      <button class="btn primary" onclick={openAnother}><Icon name="folder" size={14} /> Open another</button>
    </div>
  {:else if encoder.phase === "cancelled"}
    <div class="result"><span class="muted">Encoding cancelled. Nothing was saved.</span></div>
  {:else}
    <div class="result fail">
      <span class="badge"><Icon name="alert" size={15} /></span>
      <div class="rt"><div class="name">Export failed</div><div class="err">{encoder.error ?? "Unknown error"}</div></div>
    </div>
    {#if encoder.result?.stderrTail}
      <button class="log-toggle" aria-expanded={showLog} onclick={() => (showLog = !showLog)}>
        <Icon name="chevron" size={13} class={showLog ? "open" : ""} /> ffmpeg output
      </button>
      {#if showLog}<pre>{encoder.result.stderrTail}</pre>{/if}
    {/if}
    <div class="actions">
      <button class="btn" onclick={() => encoder.dismiss()}>Back to editing</button>
      <button class="btn primary" onclick={() => { encoder.dismiss(); process(); }}><Icon name="refresh" size={14} /> Try again</button>
    </div>
  {/if}
</div>

<style>
  .pb {
    flex: none; display: flex; flex-wrap: wrap; align-items: center; gap: 10px 12px; padding: 12px 16px 14px;
    border-top: 1px solid var(--border); background: var(--panel);
    box-shadow: 0 -8px 16px -12px rgb(0 0 0 / .35);
  }
  .out { flex: 1; min-width: 0; }
  .name { font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; user-select: text; }
  .meta { display: flex; gap: 10px; font-size: 12px; color: var(--text-2); }
  .go {
    height: 38px; padding: 0 22px; border-radius: var(--r-3); border: 0; font: 600 14px/1 var(--font-display);
    background: var(--accent); color: var(--accent-ink); transition: background var(--fast), transform var(--fast);
  }
  .go:hover:not(:disabled) { background: var(--accent-hover); }
  .go:active:not(:disabled) { transform: translateY(1px); }
  .go:disabled { opacity: .45; }
  .prog { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
  .top { display: flex; align-items: baseline; gap: 8px; font-size: 12px; }
  .big { font: 600 18px/1 var(--font-display); }
  .spacer { flex: 1; }
  .bar { height: 6px; border-radius: 999px; background: var(--surface-3); overflow: hidden; }
  .bar div { height: 100%; background: var(--accent); border-radius: 999px; transition: width 200ms linear; }
  .stats { display: flex; gap: 12px; font-size: 11.5px; min-width: 0; white-space: nowrap; }
  .file { margin-left: auto; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
  .cancel { height: 32px; }
  .result { display: flex; align-items: center; gap: 10px; width: 100%; min-width: 0; }
  .badge { flex: none; width: 28px; height: 28px; border-radius: 50%; display: grid; place-items: center; }
  .ok .badge { background: var(--success-soft); color: var(--success); }
  .fail .badge { background: var(--danger-soft); color: var(--danger); align-self: flex-start; }
  .rt { flex: 1; min-width: 0; }
  .err { font-size: 12px; color: var(--text-2); user-select: text; overflow-wrap: anywhere; }
  .actions { display: flex; gap: 8px; width: 100%; justify-content: flex-end; }
  .log-toggle { display: flex; align-items: center; gap: 4px; border: 0; background: none; color: var(--text-2); font-size: 12px; padding: 0; }
  .log-toggle :global(svg) { transform: rotate(-90deg); transition: transform var(--fast); }
  .log-toggle :global(svg.open) { transform: none; }
  pre {
    width: 100%; margin: 0; max-height: 160px; overflow: auto; padding: 8px 10px; border-radius: var(--r-2);
    background: var(--surface-2); font: 11px/1.5 Consolas, "Cascadia Mono", monospace; color: var(--text-2);
    white-space: pre-wrap; user-select: text;
  }
</style>
