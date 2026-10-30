<script lang="ts">
  // One player API regardless of source: plays the original file when the browser can decode
  // it, otherwise (or on decode failure) the backend's on-demand HLS stream via lazy hls.js.
  // All seeks are clamped to the cut; playback loops inside it.
  import { onDestroy, onMount } from "svelte";
  import type Hls from "hls.js/light";
  import { frameSrc } from "../../lib/api";
  import type { OpenedVideo } from "../../lib/types";
  import { editor, type PlayerApi } from "../../lib/state/editor.svelte";
  import { cut } from "../../lib/state/cut.svelte";
  import { crop } from "../../lib/state/crop.svelte";
  import { canPlayDirect } from "../../lib/util/codec";

  let { video }: { video: OpenedVideo } = $props();

  let el = $state<HTMLVideoElement>();
  let hls: Hls | null = null;
  let stillSrc = $state<string | null>(null);
  let stillVisible = $state(false);
  let current: OpenedVideo | null = null;
  let needsInitialSeek = false;
  let loadToken = 0;
  let watchdog: ReturnType<typeof setTimeout> | undefined;
  let pendingPreview: number | null = null;
  let stillBusy = false;
  let stillPending: number | null = null;
  let rvfcId = 0;
  let rafId = 0;

  const eps = () => 0.5 / (video.info.fps || 30);

  // --- source selection ---------------------------------------------------------------
  async function load(v: OpenedVideo) {
    if (!el) return;
    destroyHls();
    loadToken++;
    needsInitialSeek = true;
    stillVisible = false;
    stillSrc = null;
    editor.buffering = true;
    // VITE_FORCE_STREAM=1 (dev) always uses the stream, to test that path with any file.
    if (import.meta.env.VITE_FORCE_STREAM !== "1" && canPlayDirect(v.info, v.directLikelyPlayable, el)) {
      editor.source = "direct";
      el.src = v.directUrl;
      el.load();
    } else {
      await useStream(0);
    }
  }

  async function useStream(at: number) {
    if (!el || (editor.source === "stream" && hls)) return;
    const token = loadToken;
    const wasPlaying = editor.playing;
    editor.source = "stream";
    clearTimeout(watchdog);
    el.removeAttribute("src");
    el.load();
    const { default: HlsCtor } = await import("hls.js/light");
    if (!el || token !== loadToken) return;
    const h = new HlsCtor({
      startPosition: at,
      enableWorker: false,
      maxBufferLength: 16,
      maxMaxBufferLength: 32,
      backBufferLength: 30,
      // Segments may take a moment to be produced on demand.
      fragLoadPolicy: { default: { maxTimeToFirstByteMs: 30000, maxLoadTimeMs: 60000, timeoutRetry: { maxNumRetry: 3, retryDelayMs: 300, maxRetryDelayMs: 2000 }, errorRetry: { maxNumRetry: 4, retryDelayMs: 500, maxRetryDelayMs: 4000 } } },
    });
    hls = h;
    let mediaRecoveries = 0;
    h.on(HlsCtor.Events.ERROR, (_e, d) => {
      if (!d.fatal) return;
      if (d.type === HlsCtor.ErrorTypes.NETWORK_ERROR) h.startLoad(el?.currentTime ?? at);
      else if (d.type === HlsCtor.ErrorTypes.MEDIA_ERROR && mediaRecoveries++ < 3) h.recoverMediaError();
      else console.error("hls fatal", d);
    });
    h.loadSource(video.hlsUrl);
    h.attachMedia(el);
    if (wasPlaying) el.play().catch(() => {});
  }

  function destroyHls() {
    hls?.destroy();
    hls = null;
  }

  /** Direct playback failed to decode — switch to the stream at the same position. */
  function fallback(reason: string) {
    if (editor.source !== "direct") return;
    console.info("direct playback failed, streaming instead:", reason);
    void useStream(el?.currentTime || editor.time || cut.start);
  }

  // --- time / clamping ---------------------------------------------------------------
  function onFrame(t: number) {
    if (!el || editor.scrubTime != null) return;
    if (editor.playing) {
      if (t >= cut.end - eps() || t < cut.start - 0.25) {
        el.currentTime = cut.start; // loop inside the cut
        editor.time = cut.start;
        return;
      }
    }
    editor.time = cut.clampTime(t);
  }

  function startClock() {
    if (!el) return;
    stopClock();
    if ("requestVideoFrameCallback" in el) {
      const step = (_now: number, meta: VideoFrameCallbackMetadata) => {
        onFrame(meta.mediaTime);
        rvfcId = el!.requestVideoFrameCallback(step);
      };
      rvfcId = el.requestVideoFrameCallback(step);
    }
    // rAF backstop: rVFC can stall (e.g. audio-only moments); also catches the cut end promptly.
    const raf = () => { if (el && !el.paused) onFrame(el.currentTime); rafId = requestAnimationFrame(raf); };
    rafId = requestAnimationFrame(raf);
  }

  function stopClock() {
    if (el && rvfcId) el.cancelVideoFrameCallback(rvfcId);
    cancelAnimationFrame(rafId);
    rvfcId = rafId = 0;
  }

  function seekRaw(t: number) {
    if (!el) return;
    el.currentTime = t;
  }

  // --- scrub preview (stream path): coalesced stills, one request in flight ---------------
  function requestStill(t: number) {
    stillPending = t;
    if (!stillBusy) nextStill();
  }

  function nextStill() {
    if (stillPending == null) { stillBusy = false; return; }
    const t = stillPending;
    stillPending = null;
    stillBusy = true;
    const img = new Image();
    img.decoding = "async";
    img.onload = () => {
      if (editor.scrubTime != null) { stillSrc = img.src; stillVisible = true; }
      nextStill();
    };
    img.onerror = () => nextStill();
    img.src = frameSrc(video.frameUrl, t, 640, true);
  }

  const api: PlayerApi = {
    get el() { return el ?? null; },
    play() {
      if (!el) return;
      if (el.currentTime >= cut.end - eps() || el.currentTime < cut.start) seekRaw(cut.start);
      el.play().catch(() => {});
    },
    pause() { el?.pause(); },
    toggle() { if (el?.paused) api.play(); else api.pause(); },
    seek(t: number) {
      const c = cut.clampTime(t);
      editor.time = c;
      seekRaw(c);
    },
    preview(t: number) {
      const c = cut.clampTime(t);
      editor.scrubTime = t;
      if (!el) return;
      if (editor.source === "direct") {
        // Coalesce: issue the next seek only when the previous one has landed.
        if (el.seeking) { pendingPreview = c; return; }
        if (el.fastSeek && Math.abs(c - el.currentTime) > 2) el.fastSeek(c);
        else el.currentTime = c;
      } else {
        requestStill(t);
      }
    },
    endPreview(t: number) {
      pendingPreview = null;
      stillPending = null;
      editor.scrubTime = null;
      api.seek(t);
      if (editor.source === "direct") stillVisible = false;
    },
    stepFrames(n: number) {
      api.pause();
      api.seek(editor.time + n / (video.info.fps || 30));
    },
  };

  // --- element events -------------------------------------------------------------
  function onSeeked() {
    if (pendingPreview != null && el) {
      const t = pendingPreview;
      pendingPreview = null;
      el.currentTime = t;
      return;
    }
    if (editor.scrubTime == null) {
      stillVisible = false;
      if (el && el.paused) editor.time = cut.clampTime(el.currentTime);
    }
  }

  function onLoadedMetadata() {
    if (!el) return;
    editor.buffering = false;
    if (editor.source === "direct" && video.info.width > 0 && el.videoWidth === 0) {
      fallback("no video track decoded");
      return;
    }
    // First load of a file starts at the cut start; after a fallback hls.js resumes at startPosition.
    if (needsInitialSeek) {
      needsInitialSeek = false;
      if (editor.source === "direct") seekRaw(cut.start);
    }
  }

  function onPlaying() {
    editor.playing = true;
    editor.buffering = false;
    startClock();
    if (editor.source === "direct") {
      clearTimeout(watchdog);
      watchdog = setTimeout(() => {
        if (!el || el.paused || editor.source !== "direct") return;
        const q = el.getVideoPlaybackQuality?.();
        if (q && q.totalVideoFrames === 0 && video.info.width > 0) fallback("no frames decoded");
      }, 2500);
    }
  }

  function onPause() {
    editor.playing = false;
    stopClock();
    clearTimeout(watchdog);
    if (el && editor.scrubTime == null) editor.time = cut.clampTime(el.currentTime);
  }

  function onEnded() {
    // Cut end == file end: loop back.
    if (!el) return;
    seekRaw(cut.start);
    el.play().catch(() => {});
  }

  function onError() {
    const code = el?.error?.code;
    if (editor.source === "direct" && code && code >= 3) fallback(`media error ${code}`);
    else if (editor.source === "direct" && code === 2) fallback("network error on direct file");
  }

  // Sync volume / mute / rate.
  $effect(() => { if (el) el.volume = editor.volume; });
  $effect(() => { if (el) el.muted = editor.muted; });
  $effect(() => { if (el) el.playbackRate = editor.rate; });

  // New file -> reload.
  $effect(() => {
    const v = video;
    if (el && v !== current) {
      current = v;
      void load(v);
    }
  });

  // Keep the paused playhead inside the cut when the handles move past it.
  $effect(() => {
    const s = cut.start, e = cut.end;
    if (!el || editor.scrubTime != null || editor.playing) return;
    const t = editor.time;
    if (t < s || t > e) api.seek(t < s ? s : e);
  });

  // Preview-cropped uses object-view-box on the video itself.
  const viewBox = $derived.by(() => {
    const c = crop.spec;
    if (!editor.previewCropped || !c || editor.mode === "crop") return undefined;
    const W = video.info.width, H = video.info.height;
    const p = (n: number, d: number) => `${((n / d) * 100).toFixed(4)}%`;
    return `inset(${p(c.y, H)} ${p(W - c.x - c.width, W)} ${p(H - c.y - c.height, H)} ${p(c.x, W)})`;
  });

  onMount(() => { editor.player = api; });
  onDestroy(() => {
    stopClock();
    clearTimeout(watchdog);
    destroyHls();
    if (editor.player === api) editor.player = null;
  });
</script>

<!-- svelte-ignore a11y_media_has_caption -->
<video
  bind:this={el}
  preload="auto"
  playsinline
  disablepictureinpicture
  style:object-view-box={viewBox}
  onloadedmetadata={onLoadedMetadata}
  onplaying={onPlaying}
  onpause={onPause}
  onended={onEnded}
  onerror={onError}
  onseeked={onSeeked}
  onwaiting={() => (editor.buffering = true)}
  oncanplay={() => (editor.buffering = false)}
  onclick={() => api.toggle()}
></video>
{#if stillSrc}
  <img class="still" class:visible={stillVisible} src={stillSrc} alt="" draggable="false" />
{/if}

<style>
  video, .still {
    position: absolute; inset: 0; width: 100%; height: 100%; display: block;
    object-fit: fill; background: #000;
  }
  .still { opacity: 0; pointer-events: none; }
  .still.visible { opacity: 1; }
</style>
