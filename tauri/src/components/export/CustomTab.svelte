<script lang="ts">
  import { exporter } from "../../lib/state/export.svelte";
  import type { AudioCodec, PixelFormat, RateControl, SpeedPreset, VideoCodec } from "../../lib/types";
  import Segmented from "../ui/Segmented.svelte";

  const s = $derived(exporter.custom);
  const crfMax = $derived(s.codec === "av1" || s.codec === "vp9" ? 63 : 51);
  const SPEEDS: SpeedPreset[] = ["ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"];
  const containers = $derived(
    s.codec === "vp9" ? ["webm", "mkv"] : s.codec === "av1" ? ["mp4", "mkv", "webm"] : ["mp4", "mkv", "mov"],
  );

  const codecs: { value: VideoCodec; label: string; tip: string }[] = [
    { value: "h264", label: "H.264", tip: "Plays everywhere" },
    { value: "h265", label: "H.265", tip: "Smaller files; not every device plays it" },
    { value: "av1", label: "AV1", tip: "Smallest files; slow to encode; needs recent players" },
    { value: "vp9", label: "VP9", tip: "Royalty-free, good for the web" },
  ];
  const rcs: { value: RateControl; label: string }[] = [
    { value: "crf", label: "Quality" }, { value: "vbr", label: "Bitrate" }, { value: "targetSize", label: "File size" },
  ];
  const audios: { value: AudioCodec; label: string }[] = [
    { value: "aac", label: "AAC" }, { value: "opus", label: "Opus" }, { value: "mp3", label: "MP3" }, { value: "copy", label: "Copy" },
  ];
  const pix: { value: PixelFormat; label: string; tip: string }[] = [
    { value: "yuv420p", label: "8-bit", tip: "Most compatible" },
    { value: "yuv420p10le", label: "10-bit", tip: "Less banding in gradients; fewer devices decode it" },
  ];

  function setCodec(c: VideoCodec) {
    const max = c === "av1" || c === "vp9" ? 63 : 51;
    const defaults: Record<VideoCodec, number> = { h264: 23, h265: 26, av1: 32, vp9: 33 };
    const crf = s.codec === c ? s.crf : Math.min(max, defaults[c]);
    const container = s.container && !(c === "vp9" ? ["webm", "mkv"] : c === "av1" ? ["mp4", "mkv", "webm"] : ["mp4", "mkv", "mov"]).includes(s.container) ? null : s.container;
    exporter.patchCustom({ codec: c, crf, container });
  }

  const crfHint = $derived(
    s.crf <= (crfMax === 51 ? 18 : 24) ? "Near lossless, large" : s.crf <= (crfMax === 51 ? 24 : 34) ? "Balanced" : s.crf <= (crfMax === 51 ? 30 : 44) ? "Smaller, softer" : "Very small, visibly degraded",
  );
</script>

<div class="rows">
  <div class="row"><span class="label">Codec</span>
    <Segmented size="sm" label="Video codec" value={s.codec} options={codecs} onchange={setCodec} />
  </div>

  <div class="row"><span class="label">Target</span>
    <Segmented size="sm" label="Rate control" value={s.rateControl === "cbr" ? "vbr" : s.rateControl} options={rcs} onchange={(v) => exporter.patchCustom({ rateControl: v })} />
  </div>

  {#if s.rateControl === "crf"}
    <div class="row">
      <div class="between"><span class="label">CRF <b class="tnum">{s.crf}</b></span><span class="faint hint">{crfHint}</span></div>
      <input type="range" min="0" max={crfMax} step="1" value={s.crf} aria-label="CRF"
        style:--v="{(s.crf / crfMax) * 100}%" oninput={(e) => exporter.patchCustom({ crf: +e.currentTarget.value })} />
      <div class="between faint ends"><span>Best quality</span><span>Smallest</span></div>
    </div>
  {:else if s.rateControl === "targetSize"}
    <label class="row"><span class="label">File size (MB)</span>
      <input class="field" type="number" min="1" step="1" value={Math.round((s.targetSizeBytes ?? 50 * 1048576) / 1048576)}
        onchange={(e) => exporter.patchCustom({ targetSizeBytes: Math.max(1, +e.currentTarget.value) * 1048576 })} />
    </label>
  {:else}
    <label class="row"><span class="label">Video bitrate (kbps)</span>
      <input class="field" type="number" min="100" step="100" value={Math.round((s.targetBitrateBps ?? 5e6) / 1000)}
        onchange={(e) => exporter.patchCustom({ targetBitrateBps: Math.max(100, +e.currentTarget.value) * 1000 })} />
    </label>
  {/if}

  <div class="two">
    <label class="row"><span class="label">Speed</span>
      <select class="field" value={s.speed} onchange={(e) => exporter.patchCustom({ speed: e.currentTarget.value as SpeedPreset })}>
        {#each SPEEDS as sp (sp)}<option value={sp}>{sp}</option>{/each}
      </select>
    </label>
    <label class="row"><span class="label">Container</span>
      <select class="field" value={s.container ?? ""} onchange={(e) => exporter.patchCustom({ container: e.currentTarget.value || null })}>
        <option value="">Automatic</option>
        {#each containers as c (c)}<option value={c}>.{c}</option>{/each}
      </select>
    </label>
  </div>

  <div class="row"><span class="label">Bit depth</span>
    <Segmented size="sm" label="Pixel format" value={s.pixelFormat} options={pix} onchange={(v) => exporter.patchCustom({ pixelFormat: v })} />
  </div>

  <div class="row"><span class="label">Audio</span>
    <Segmented size="sm" label="Audio codec" value={s.audioCodec} options={audios} onchange={(v) => exporter.patchCustom({ audioCodec: v })} />
  </div>
  {#if s.audioCodec !== "copy"}
    <div class="row">
      <span class="label">Audio bitrate <b class="tnum">{s.audioBitrateKbps} kbps</b></span>
      <input type="range" min="64" max="320" step="32" value={s.audioBitrateKbps} aria-label="Audio bitrate"
        style:--v="{((s.audioBitrateKbps - 64) / 256) * 100}%" oninput={(e) => exporter.patchCustom({ audioBitrateKbps: +e.currentTarget.value })} />
    </div>
  {/if}
</div>

<style>
  .rows { display: flex; flex-direction: column; gap: 10px; }
  .row { display: flex; flex-direction: column; gap: 5px; }
  .two { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .between { display: flex; justify-content: space-between; align-items: baseline; }
  .label b { color: var(--text); font-weight: 600; margin-left: 4px; }
  .hint { font-size: 11.5px; }
  .ends { font-size: 11px; }
  input[type="range"] {
    -webkit-appearance: none; appearance: none; width: 100%; height: 4px; border-radius: 4px; margin: 6px 0;
    background: linear-gradient(to right, var(--accent) var(--v), var(--surface-3) var(--v));
  }
  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none; width: 14px; height: 14px; border-radius: 50%; background: var(--surface);
    border: 2px solid var(--accent); box-shadow: var(--shadow-1);
  }
  select.field { width: 100%; }
</style>
