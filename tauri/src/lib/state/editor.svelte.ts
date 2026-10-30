import type { OpenedVideo } from "../types";

/** What the Player component exposes; the rest of the app never touches <video> directly. */
export interface PlayerApi {
  play(): void;
  pause(): void;
  toggle(): void;
  /** Real seek, clamped to the cut. */
  seek(t: number): void;
  /** Lightweight preview while scrubbing (fastSeek on direct, still frames on stream). */
  preview(t: number): void;
  /** Ends a scrub: hides preview stills and seeks for real to `t`. */
  endPreview(t: number): void;
  stepFrames(n: number): void;
  readonly el: HTMLVideoElement | null;
}

export type Source = "direct" | "stream";

class EditorStore {
  video = $state<OpenedVideo | null>(null);
  opening = $state(false);
  openError = $state<string | null>(null);

  // Playback (mirrored from the Player).
  time = $state(0);
  playing = $state(false);
  source = $state<Source>("direct");
  buffering = $state(false);
  volume = $state(1);
  muted = $state(false);
  rate = $state(1);
  /** Time shown while scrubbing (playhead follows it before the real seek lands). */
  scrubTime = $state<number | null>(null);

  mode = $state<"edit" | "crop">("edit");
  previewCropped = $state(false);

  player: PlayerApi | null = null;

  get info() { return this.video?.info ?? null; }
  get fps() { return this.video?.info.fps || 30; }
  get displayTime() { return this.scrubTime ?? this.time; }
}

export const editor = new EditorStore();
