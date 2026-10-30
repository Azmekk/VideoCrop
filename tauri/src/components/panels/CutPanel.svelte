<script lang="ts">
  import { editor } from "../../lib/state/editor.svelte";
  import { cut } from "../../lib/state/cut.svelte";
  import { formatDuration } from "../../lib/util/time";
  import Section from "./Section.svelte";
  import TimeInput from "../ui/TimeInput.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import Icon from "../ui/Icon.svelte";

  const long = $derived((editor.info?.durationSec ?? 0) >= 3600);
  const status = $derived(cut.isFull ? "Whole video" : `${formatDuration(cut.length)} of ${formatDuration(cut.duration)}`);

  function setStart(t: number) { cut.setStart(t); editor.player?.seek(cut.start); }
  function setEnd(t: number) { cut.setEnd(t); editor.player?.seek(cut.end); }
</script>

<Section title="Cut" icon="cut" {status} active={!cut.isFull}>
  {#snippet actions()}
    <button class="btn ghost sm" disabled={cut.isFull} onclick={() => cut.reset()} data-tip="Keep the whole video"><Icon name="reset" size={13} /> Reset</button>
  {/snippet}

  <div class="grid">
    <div class="lbl label">Start</div>
    <TimeInput label="Cut start" value={cut.start} onchange={setStart} withHours={long} />
    <button class="btn sm icon" onclick={() => cut.setStart(editor.displayTime)} aria-label="Set start from playhead" data-tip="Set start from playhead (I)"><Icon name="markIn" size={14} /></button>

    <div class="lbl label">End</div>
    <TimeInput label="Cut end" value={cut.end} onchange={setEnd} withHours={long} />
    <button class="btn sm icon" onclick={() => cut.setEnd(editor.displayTime)} aria-label="Set end from playhead" data-tip="Set end from playhead (O)"><Icon name="markOut" size={14} /></button>
  </div>

  <Segmented label="Cut precision" value={cut.accurate ? "accurate" : "fast"}
    options={[{ value: "fast", label: "Fast" }, { value: "accurate", label: "Accurate" }]}
    onchange={(v) => (cut.accurate = v === "accurate")} />
  <p class="note">
    {cut.accurate
      ? "Starts on the exact frame. Slower, because the video is decoded up to the start point."
      : "Starts at the nearest keyframe, so it may begin up to a few seconds early. Quick."}
  </p>
</Section>

<style>
  .grid { display: grid; grid-template-columns: 44px 1fr auto; gap: 6px 8px; align-items: center; }
  .grid :global(.btn.icon) { width: 28px; height: 28px; }
</style>
