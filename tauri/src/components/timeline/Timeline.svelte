<script lang="ts">
  import { untrack } from "svelte";
  import { editor } from "../../lib/state/editor.svelte";
  import { cut, MIN_CUT } from "../../lib/state/cut.svelte";
  import { timeline } from "../../lib/state/timeline.svelte";
  import { resetThumbs } from "../../lib/state/frames.svelte";
  import { displaySize } from "../../lib/util/geometry";
  import { clamp, formatTime } from "../../lib/util/time";
  import { scrubRate, scrubStep, snapToFrame } from "../../lib/util/scrub";
  import { timeToX, xToTime } from "../../lib/util/zoom";
  import Filmstrip from "./Filmstrip.svelte";
  import Ruler from "./Ruler.svelte";
  import Overview from "./Overview.svelte";
  import Icon from "../ui/Icon.svelte";

  type Kind = "playhead" | "start" | "end" | "pan";
  interface Drag { kind: Kind; lastX: number; value: number; offset: number; factor: number; wasPlaying: boolean }

  let width = $state(0);
  let trackEl = $state<HTMLDivElement>();
  let laneEl = $state<HTMLDivElement>();
  let drag: Drag | null = null;
  let dragging = $state(false);
  let pill = $state<{ label: string; x: number; t: number } | null>(null);

  const info = $derived(editor.info!);
  const dur = $derived(info.durationSec);
  const fps = $derived(editor.fps);
  const view = $derived(timeline.view);
  const aspect = $derived.by(() => { const d = displaySize(info); return d.width / d.height; });
  const X = (t: number) => timeToX(t, view, width);
  const xs = $derived({ s: X(cut.start), e: X(cut.end), p: X(editor.displayTime) });
  const zoomLabel = $derived(dur / view.span);
  const long = $derived(dur >= 3600);

  // New file: reset view + thumbnails.
  $effect(() => {
    const v = editor.video;
    if (!v) return;
    untrack(() => { timeline.fitAll(); resetThumbs(v.frameUrl); });
  });

  // Page the view to follow playback when zoomed.
  $effect(() => {
    const t = editor.time;
    if (editor.playing && !dragging) untrack(() => timeline.follow(t));
  });

  const timeAt = (clientX: number) => {
    const r = trackEl!.getBoundingClientRect();
    return xToTime(clientX - r.left, view, width);
  };

  function begin(e: PointerEvent, kind: Kind, value: number) {
    e.preventDefault();
    e.stopPropagation();
    trackEl!.setPointerCapture(e.pointerId);
    const wasPlaying = editor.playing;
    if (kind !== "pan") editor.player?.pause();
    drag = { kind, lastX: e.clientX, value, offset: value - timeAt(e.clientX), factor: 1, wasPlaying };
    dragging = true;
    if (kind !== "pan") apply(value);
  }

  function apply(v: number) {
    const p = editor.player;
    if (!drag || !p) return;
    if (drag.kind === "playhead") p.preview(cut.clampTime(v));
    else if (drag.kind === "start") { cut.setStart(v); p.preview(cut.start); }
    else if (drag.kind === "end") { cut.setEnd(v); p.preview(cut.end); }
  }

  function onTrackDown(e: PointerEvent) {
    if (e.button === 1) { begin(e, "pan", 0); return; }
    if (e.button !== 0) return;
    // Clicking outside the cut snaps to the nearest edge.
    begin(e, "playhead", cut.clampTime(timeAt(e.clientX)));
  }

  function onMove(e: PointerEvent) {
    if (!drag || !trackEl) return;
    const dx = e.clientX - drag.lastX;
    drag.lastX = e.clientX;
    if (drag.kind === "pan") { timeline.panBy(-(dx / width) * view.span); return; }

    const r = trackEl.getBoundingClientRect();
    const dy = e.clientY < r.top ? r.top - e.clientY : e.clientY > r.bottom ? e.clientY - r.bottom : 0;
    const rate = scrubRate(dy, e.altKey);
    if (rate.factor !== drag.factor) {
      // Re-anchor so switching speed never makes the value jump.
      drag.offset = drag.value - timeAt(e.clientX);
      drag.factor = rate.factor;
    }
    drag.value = clamp(rate.factor === 1 ? timeAt(e.clientX) + drag.offset : scrubStep(drag.value, dx, view.span / width, rate), 0, dur);
    const v = rate.frameSnap ? snapToFrame(drag.value, fps) : drag.value;
    apply(v);
    const shown = drag.kind === "start" ? cut.start : drag.kind === "end" ? cut.end : cut.clampTime(v);
    pill = rate.factor < 1 ? { label: rate.label, x: clamp(X(shown), 60, width - 60), t: shown } : null;
  }

  function onUp() {
    if (!drag) return;
    const d = drag;
    drag = null;
    dragging = false;
    pill = null;
    if (d.kind === "pan") return;
    const p = editor.player;
    if (!p) return;
    const final = d.kind === "start" ? cut.start : d.kind === "end" ? cut.end : cut.clampTime(d.value);
    p.endPreview(final);
    if (d.wasPlaying && d.kind === "playhead") p.play();
  }

  function onHandleKey(e: KeyboardEvent, which: "start" | "end") {
    const step = e.shiftKey ? 1 : 1 / fps;
    const d = e.key === "ArrowLeft" || e.key === "ArrowDown" ? -step : e.key === "ArrowRight" || e.key === "ArrowUp" ? step : 0;
    if (!d) return;
    e.preventDefault();
    e.stopPropagation();
    if (which === "start") { cut.setStart(cut.start + d); editor.player?.seek(cut.start); }
    else { cut.setEnd(cut.end + d); editor.player?.seek(cut.end); }
  }

  // Wheel: zoom around the cursor; Shift+wheel or horizontal wheel pans. Non-passive so we can preventDefault.
  $effect(() => {
    const el = laneEl;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const unit = e.deltaMode === 1 ? 16 : 1;
      const horiz = Math.abs(e.deltaX) > Math.abs(e.deltaY);
      if (e.shiftKey || horiz) {
        const d = (horiz ? e.deltaX : e.deltaY) * unit;
        timeline.panBy((d / Math.max(1, width)) * view.span);
      } else {
        const r = trackEl!.getBoundingClientRect();
        timeline.zoomBy(Math.exp(e.deltaY * unit * 0.0016), xToTime(e.clientX - r.left, view, width));
      }
    };
    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  });

  const handleText = (t: number) => formatTime(t, long);
</script>

<div class="timeline">
  <div class="head">
    <div class="tools">
      <button class="btn ghost sm icon" onclick={() => timeline.zoomBy(1.6)} disabled={!timeline.zoomed} aria-label="Zoom out" data-tip="Zoom out (−)"><Icon name="zoomOut" size={15} /></button>
      <button class="btn ghost sm icon" onclick={() => timeline.zoomBy(1 / 1.6)} disabled={view.span <= timeline.minSpan + 1e-6} aria-label="Zoom in" data-tip="Zoom in (+)"><Icon name="zoomIn" size={15} /></button>
      <button class="btn ghost sm" onclick={() => timeline.zoomToCut()} disabled={cut.isFull} data-tip="Fit the cut to the timeline"><Icon name="fitCut" size={14} /> Zoom to cut</button>
      <button class="btn ghost sm" onclick={() => timeline.fitAll()} disabled={!timeline.zoomed} data-tip="Show the whole video"><Icon name="fitAll" size={14} /> Show all</button>
      {#if timeline.zoomed}<span class="zoom tnum">{zoomLabel < 10 ? zoomLabel.toFixed(1) : Math.round(zoomLabel)}× zoom</span>{/if}
    </div>
    <span class="hint">Scroll to zoom. Shift+scroll to pan. Drag away from the track to scrub finely.</span>
  </div>

  <div class="lane" bind:this={laneEl}>
    <div class="inner" bind:clientWidth={width}>
      <Ruler {view} {width} {fps} />

      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="track" bind:this={trackEl} class:dragging
        onpointerdown={onTrackDown} onpointermove={onMove} onpointerup={onUp} onpointercancel={onUp}
        onauxclick={(e) => e.preventDefault()}
      >
        <Filmstrip {view} {width} height={52} duration={dur} {fps} {aspect} />
        <div class="dim" style:left="0" style:width="{clamp(xs.s, 0, width)}px"></div>
        <div class="dim" style:left="{clamp(xs.e, 0, width)}px" style:right="0"></div>
        <div class="range" style:left="{xs.s}px" style:width="{Math.max(0, xs.e - xs.s)}px"></div>

        {#if xs.s >= -12 && xs.s <= width}
          <div class="handle start" style:left="{xs.s}px" role="slider" tabindex="0"
            aria-label="Cut start" aria-valuemin={0} aria-valuemax={+(cut.end - MIN_CUT).toFixed(3)} aria-valuenow={+cut.start.toFixed(3)}
            aria-valuetext={handleText(cut.start)} data-tip="Cut start (I). Drag, or use ←/→"
            onpointerdown={(e) => e.button === 0 && begin(e, "start", cut.start)} onkeydown={(e) => onHandleKey(e, "start")}
          ><i></i></div>
        {/if}
        {#if xs.e >= 0 && xs.e <= width + 12}
          <div class="handle end" style:left="{xs.e}px" role="slider" tabindex="0"
            aria-label="Cut end" aria-valuemin={+(cut.start + MIN_CUT).toFixed(3)} aria-valuemax={+dur.toFixed(3)} aria-valuenow={+cut.end.toFixed(3)}
            aria-valuetext={handleText(cut.end)} data-tip="Cut end (O). Drag, or use ←/→"
            onpointerdown={(e) => e.button === 0 && begin(e, "end", cut.end)} onkeydown={(e) => onHandleKey(e, "end")}
          ><i></i></div>
        {/if}

        {#if xs.p >= 0 && xs.p <= width}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="playhead" style:left="{xs.p}px"
            onpointerdown={(e) => e.button === 0 && begin(e, "playhead", editor.displayTime)}><span class="knob"></span></div>
        {/if}

        {#if pill}
          <div class="pill tnum" style:left="{pill.x}px"><b>{pill.label}</b> {formatTime(pill.t, long)}</div>
        {/if}
      </div>

      <Overview />
    </div>
  </div>
</div>

<style>
  .timeline {
    flex: none; background: var(--panel); border-top: 1px solid var(--border);
    padding: 4px 0 10px;
  }
  .head { display: flex; align-items: center; gap: 10px; padding: 0 10px; height: 30px; }
  .tools { display: flex; align-items: center; gap: 2px; }
  .zoom { margin-left: 8px; font-size: 11.5px; color: var(--text-3); }
  .hint { margin-left: auto; font-size: 11.5px; color: var(--text-3); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; }
  .lane { padding: 0 22px; }
  .inner { position: relative; }
  .track {
    position: relative; height: 52px; border-radius: var(--r-1); background: var(--track);
    cursor: pointer; touch-action: none;
  }
  .track.dragging { cursor: grabbing; }
  .dim { position: absolute; top: 0; bottom: 0; background: var(--filmstrip-dim); pointer-events: none; }
  .range {
    position: absolute; top: -1px; bottom: -1px; pointer-events: none;
    border-top: 2px solid var(--accent); border-bottom: 2px solid var(--accent);
    box-shadow: inset 0 0 0 9999px rgb(245 184 61 / .06);
  }
  .handle {
    position: absolute; top: -1px; bottom: -1px; width: 12px; background: var(--accent);
    cursor: ew-resize; display: grid; place-items: center; z-index: 2;
    transition: background var(--fast);
  }
  .handle.start { margin-left: -12px; border-radius: 5px 0 0 5px; }
  .handle.end { border-radius: 0 5px 5px 0; }
  .handle i { width: 2px; height: 16px; border-radius: 1px; background: var(--accent-ink); opacity: .55; }
  .handle:hover, .handle:focus-visible { background: var(--accent-hover); }
  .handle:focus-visible { outline: 2px solid var(--focus); outline-offset: 1px; }
  .playhead {
    position: absolute; top: -22px; bottom: -4px; width: 2px; margin-left: -1px; z-index: 3;
    background: var(--text); cursor: grab; box-shadow: 0 0 0 1px rgb(0 0 0 / .25);
  }
  .playhead::after { content: ""; position: absolute; inset: 0 -6px; } /* hit area */
  .knob {
    position: absolute; top: 0; left: 50%; width: 11px; height: 11px; transform: translateX(-50%);
    background: var(--text); clip-path: polygon(0 0, 100% 0, 100% 55%, 50% 100%, 0 55%); border-radius: 2px;
  }
  .pill {
    position: absolute; bottom: calc(100% + 26px); transform: translateX(-50%); z-index: 4; pointer-events: none;
    padding: 4px 9px; border-radius: 999px; font-size: 11.5px; white-space: nowrap;
    background: var(--text); color: var(--bg); box-shadow: var(--shadow-2);
  }
  .pill b { font-weight: 600; margin-right: 4px; }
  @media (max-width: 1180px) { .hint { display: none; } }
</style>
