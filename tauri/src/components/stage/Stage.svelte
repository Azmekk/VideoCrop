<script lang="ts">
  import { editor } from "../../lib/state/editor.svelte";
  import { crop } from "../../lib/state/crop.svelte";
  import { displaySize } from "../../lib/util/geometry";
  import Player from "./Player.svelte";
  import CropEditor from "./CropEditor.svelte";
  import Icon from "../ui/Icon.svelte";

  let w = $state(0);
  let h = $state(0);

  const info = $derived(editor.info!);
  const cropped = $derived(editor.previewCropped && !!crop.spec && editor.mode === "edit");

  // Content aspect in display space (SAR-aware); switches to the crop's aspect when previewing it.
  const aspect = $derived.by(() => {
    const d = cropped ? crop.displayDims : displaySize(info);
    return d.width / Math.max(1, d.height);
  });

  const PAD = 16;
  const box = $derived.by(() => {
    const aw = Math.max(0, w - PAD * 2), ah = Math.max(0, h - PAD * 2);
    let bw = aw, bh = aw / aspect;
    if (bh > ah) { bh = ah; bw = ah * aspect; }
    return { w: Math.floor(bw), h: Math.floor(bh) };
  });

  const streamTip = $derived.by(() => {
    const ext = info.path.split(".").pop()?.toUpperCase() ?? "";
    const fmt = [info.videoCodec.toUpperCase(), ext].filter(Boolean).join(" in ");
    return `The built-in player can't play this file (${fmt}) directly, so VideoCrop converts it with ffmpeg on the fly ` +
      `into a lightweight 720p preview, a few seconds ahead of playback. Jumping to a part that hasn't been converted yet ` +
      `takes a moment. This only affects the preview: exports always use the original file at full quality.`;
  });

  // Applied-crop window in % of the full frame (edit mode, not previewing).
  const win = $derived.by(() => {
    const c = crop.spec;
    if (!c || cropped || editor.mode !== "edit") return null;
    return {
      l: (c.x / info.width) * 100, t: (c.y / info.height) * 100,
      w: (c.width / info.width) * 100, h: (c.height / info.height) * 100,
    };
  });
</script>

<div class="stage" bind:clientWidth={w} bind:clientHeight={h}>
  {#if editor.video}
    <div class="box" class:cropping={editor.mode === "crop"} style:width="{box.w}px" style:height="{box.h}px">
      <Player video={editor.video} />
      {#if win}
        <div class="crop-win" style:left="{win.l}%" style:top="{win.t}%" style:width="{win.w}%" style:height="{win.h}%"></div>
      {/if}
      {#if editor.mode === "crop"}
        <CropEditor boxW={box.w} boxH={box.h} />
      {/if}
      {#if editor.buffering && editor.mode === "edit"}
        {#if editor.source === "stream"}
          <div class="processing" role="status">
            <div class="spinner inline"></div>
            <div>
              <b>Processing video…</b>
              <span>Converting this part so it can play in VideoCrop</span>
            </div>
          </div>
        {:else}
          <div class="spinner" aria-label="Loading"></div>
        {/if}
      {/if}
    </div>
  {/if}

  {#if editor.source === "stream"}
    <div class="badge">
      <Icon name="stream" size={13} /> Streaming preview
      <button type="button" class="help" aria-label="What is a streaming preview?" data-tip={streamTip}><Icon name="help" size={14} /></button>
    </div>
  {/if}
</div>

<style>
  .stage {
    position: relative; flex: 1; min-height: 0; display: grid; place-items: center;
    background: var(--stage); overflow: hidden;
  }
  .box { position: relative; overflow: hidden; background: #000; box-shadow: 0 0 0 1px rgb(255 255 255 / .04); }
  .box.cropping { overflow: visible; }
  .crop-win {
    position: absolute; pointer-events: none;
    box-shadow: 0 0 0 9999px rgb(0 0 0 / .58);
    outline: 1px solid rgb(255 255 255 / .55);
  }
  .badge {
    position: absolute; top: 10px; left: 10px; display: flex; align-items: center; gap: 6px;
    height: 24px; padding: 0 9px; border-radius: 999px; font-size: 11.5px; font-weight: 500;
    color: #e7e9ee; background: rgb(20 22 26 / .72); border: 1px solid rgb(255 255 255 / .1);
    backdrop-filter: blur(8px);
  }
  .help {
    display: grid; place-items: center; margin-right: -4px; padding: 0; border: 0; background: none;
    color: rgb(231 233 238 / .7); cursor: help; border-radius: 50%;
  }
  .help:hover, .help:focus-visible { color: #fff; }
  .help:focus-visible { outline: 2px solid var(--focus); outline-offset: 1px; }
  .spinner {
    position: absolute; left: 50%; top: 50%; width: 28px; height: 28px; margin: -14px 0 0 -14px;
    border-radius: 50%; border: 2px solid rgb(255 255 255 / .18); border-top-color: #fff;
    animation: spin .8s linear infinite; pointer-events: none;
  }
  .spinner.inline { position: static; flex: none; width: 22px; height: 22px; margin: 0; }
  .processing {
    position: absolute; left: 50%; top: 50%; transform: translate(-50%, -50%); max-width: calc(100% - 32px);
    display: flex; align-items: center; gap: 12px; padding: 12px 16px; border-radius: 10px; pointer-events: none;
    color: #e7e9ee; background: rgb(20 22 26 / .82); border: 1px solid rgb(255 255 255 / .1);
    backdrop-filter: blur(8px); box-shadow: 0 8px 24px rgb(0 0 0 / .35);
  }
  .processing b { display: block; font-size: 13px; font-weight: 600; }
  .processing span { display: block; margin-top: 2px; font-size: 11.5px; color: rgb(231 233 238 / .7); }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
