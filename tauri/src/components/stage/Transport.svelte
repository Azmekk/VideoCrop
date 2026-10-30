<script lang="ts">
  import { editor } from "../../lib/state/editor.svelte";
  import { cut } from "../../lib/state/cut.svelte";
  import { formatTime } from "../../lib/util/time";
  import Icon from "../ui/Icon.svelte";

  const RATES = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 2];
  const long = $derived((editor.info?.durationSec ?? 0) >= 3600);
  const rel = $derived(Math.max(0, editor.displayTime - cut.start));
  const p = () => editor.player;
</script>

<div class="transport">
  <div class="group">
    <button class="btn ghost icon" onclick={() => p()?.seek(cut.start)} aria-label="Jump to in point" data-tip="Jump to in point (Home)">
      <Icon name="toStart" size={15} />
    </button>
    <button class="btn ghost icon" onclick={() => p()?.stepFrames(-1)} aria-label="Previous frame" data-tip="Previous frame (←)">
      <Icon name="frameBack" size={15} />
    </button>
    <button class="play" onclick={() => p()?.toggle()} aria-label={editor.playing ? "Pause" : "Play"} data-tip={editor.playing ? "Pause (Space)" : "Play (Space)"}>
      <Icon name={editor.playing ? "pause" : "play"} size={16} />
    </button>
    <button class="btn ghost icon" onclick={() => p()?.stepFrames(1)} aria-label="Next frame" data-tip="Next frame (→)">
      <Icon name="frameFwd" size={15} />
    </button>
    <button class="btn ghost icon" onclick={() => { p()?.pause(); p()?.seek(cut.end - 1 / editor.fps); }} aria-label="Jump to out point" data-tip="Jump to out point (End)">
      <Icon name="toEnd" size={15} />
    </button>
  </div>

  <div class="time tnum" aria-live="off">
    <div class="primary"><span>{formatTime(rel, long)}</span><span class="sep">/</span><span class="muted">{formatTime(cut.length, long)}</span></div>
    <div class="secondary">at {formatTime(editor.displayTime, long)} of {formatTime(editor.info?.durationSec ?? 0, long)}</div>
  </div>

  <div class="spacer"></div>

  <label class="rate" data-tip="Playback speed">
    <span class="sr-only">Playback speed</span>
    <select class="field" value={editor.rate} onchange={(e) => (editor.rate = +e.currentTarget.value)}>
      {#each RATES as r (r)}<option value={r}>{r}×</option>{/each}
    </select>
  </label>

  <div class="vol">
    <button class="btn ghost icon" onclick={() => (editor.muted = !editor.muted)} aria-label={editor.muted ? "Unmute" : "Mute"} data-tip={editor.muted ? "Unmute" : "Mute"}>
      <Icon name={editor.muted || editor.volume === 0 ? "mute" : "volume"} size={16} />
    </button>
    <input type="range" min="0" max="1" step="0.01" aria-label="Volume"
      value={editor.muted ? 0 : editor.volume}
      style:--v="{(editor.muted ? 0 : editor.volume) * 100}%"
      oninput={(e) => { editor.volume = +e.currentTarget.value; editor.muted = editor.volume === 0; }} />
  </div>
</div>

<style>
  .transport {
    flex: none; display: flex; align-items: center; gap: 14px; padding: 6px 12px;
    background: var(--panel); border-top: 1px solid var(--border);
  }
  .group { display: flex; align-items: center; gap: 2px; }
  .play {
    width: 34px; height: 34px; margin: 0 4px; border-radius: 50%; border: 0; display: grid; place-items: center;
    background: var(--text); color: var(--bg); transition: transform var(--fast), background var(--fast);
  }
  .play:hover { background: var(--accent); color: var(--accent-ink); }
  .play:active { transform: scale(.95); }
  .time { display: flex; flex-direction: column; line-height: 1.15; }
  .primary { font: 600 19px/1.15 var(--font-display); letter-spacing: .01em; display: flex; gap: 6px; font-variant-numeric: tabular-nums; }
  .primary .muted { font-weight: 500; }
  .sep { color: var(--text-3); font-weight: 400; }
  .secondary { font-size: 11.5px; color: var(--text-3); }
  .spacer { flex: 1; }
  .rate select { height: 26px; width: 66px; font-size: 12px; }
  .vol { display: flex; align-items: center; gap: 2px; }
  input[type="range"] {
    -webkit-appearance: none; appearance: none; width: 84px; height: 4px; border-radius: 4px;
    background: linear-gradient(to right, var(--text-2) var(--v), var(--surface-3) var(--v));
  }
  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none; width: 12px; height: 12px; border-radius: 50%; background: var(--text);
    box-shadow: var(--shadow-1);
  }
  @media (max-width: 1100px) { .secondary { display: none; } }
</style>
