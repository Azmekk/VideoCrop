<script lang="ts">
  import { onMount } from "svelte";
  import { api, events } from "../lib/api";
  import Icon from "./ui/Icon.svelte";

  /** Why the ffmpeg on PATH was rejected, when there is one. */
  let { problem = null, onready }: { problem?: string | null; onready: () => void } = $props();

  let step = $state<"idle" | "downloading" | "error">("idle");
  let fraction = $state(0);
  let status = $state("");
  let error = $state<string | null>(null);

  onMount(() => {
    const un = events.onToolsProgress((p) => { fraction = p.fraction; status = p.status; });
    return () => { un.then((f) => f()); };
  });

  async function start() {
    step = "downloading";
    error = null;
    fraction = 0;
    status = "Connecting…";
    try {
      const s = await api.toolsDownload();
      if (s.ready) onready();
      else throw "The download finished but ffmpeg wasn't found. Try again.";
    } catch (e) {
      step = "error";
      error = String(e);
    }
  }
</script>

<section class="wrap">
  <div class="card">
    <div class="glyph" aria-hidden="true"><Icon name="download" size={22} /></div>
    <h1>One-time setup</h1>
    <p>
      VideoCrop uses <strong>ffmpeg</strong> to read and encode video. It's downloaded once
      (about 100–150 MB) and kept on this PC — you won't see this again.
    </p>
    {#if problem}
      <div class="why" role="note"><Icon name="info" size={15} /><span>{problem}</span></div>
    {/if}

    {#if step === "idle"}
      <button class="btn primary big" onclick={start}><Icon name="download" size={15} /> Download ffmpeg</button>
    {:else if step === "downloading"}
      <div class="progress" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(fraction * 100)}>
        <div class="fill" style:width="{fraction * 100}%"></div>
      </div>
      <div class="row tnum"><span class="muted">{status}</span><span>{Math.round(fraction * 100)}%</span></div>
    {:else}
      <div class="err" role="alert"><Icon name="alert" size={15} /><span>{error}</span></div>
      <button class="btn primary big" onclick={start}><Icon name="refresh" size={15} /> Try again</button>
    {/if}
    <p class="fine">Source: the GPL build of FFmpeg by BtbN on GitHub.</p>
  </div>
</section>

<style>
  .why {
    display: flex; gap: 8px; align-items: flex-start; text-align: left; font-size: 12.5px; line-height: 1.45;
    padding: 8px 10px; border-radius: var(--r-2); background: var(--surface-2); color: var(--text-2);
    overflow-wrap: anywhere;
  }
  .why :global(svg) { flex: none; margin-top: 1px; color: var(--accent-text); }
  .wrap { flex: 1; display: grid; place-items: center; padding: 24px; }
  .card {
    width: min(440px, 100%); padding: 28px; border-radius: var(--r-4);
    background: var(--panel); border: 1px solid var(--border); box-shadow: var(--shadow-2);
    display: flex; flex-direction: column; gap: 14px;
  }
  .glyph {
    width: 44px; height: 44px; border-radius: var(--r-3); display: grid; place-items: center;
    background: var(--accent-soft); color: var(--accent-text);
  }
  h1 { margin: 0; font: 600 20px/1.2 var(--font-display); }
  p { margin: 0; color: var(--text-2); line-height: 1.55; }
  p strong { color: var(--text); font-weight: 600; }
  .big { height: 34px; align-self: flex-start; padding: 0 14px; }
  .progress { height: 6px; border-radius: 999px; background: var(--surface-2); overflow: hidden; }
  .fill { height: 100%; background: var(--accent); transition: width 160ms linear; }
  .row { display: flex; justify-content: space-between; font-size: 12px; margin-top: -6px; }
  .err { display: flex; gap: 8px; padding: 9px 11px; border-radius: var(--r-2); background: var(--danger-soft); font-size: 12.5px; user-select: text; }
  .err :global(svg) { color: var(--danger); flex: none; margin-top: 1px; }
  .fine { font-size: 11.5px; color: var(--text-3); }
</style>
