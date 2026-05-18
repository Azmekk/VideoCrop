# VideoCrop — Implementation Plan

A Windows desktop application for compressing, cutting, resizing, and cropping videos using ffmpeg, with embedded playback via mpv.

---

## 1. Tech Stack

| Layer            | Choice                                                |
|------------------|-------------------------------------------------------|
| Runtime          | .NET 10                                               |
| Compilation      | Standard JIT, framework-dependent Release builds      |
| UI framework     | WinUI 3 (Windows App SDK 1.6+)                        |
| Packaging        | Unpackaged or MSIX (decide before release)            |
| Playback engine  | mpv (spawned as external process)                     |
| Encoding engine  | ffmpeg (spawned as external process)                  |
| Metadata         | ffprobe (spawned as external process)                 |
| MVVM             | `CommunityToolkit.Mvvm` runtime classes (`ObservableObject`, `RelayCommand` class) — no source-generator attributes |
| JSON             | `System.Text.Json` reflection-based (`JsonSerializer.Deserialize<T>`) |
| Regex            | `new Regex(pattern, RegexOptions.Compiled)` for hot paths |
| Logging          | `Microsoft.Extensions.Logging` + `Serilog.Sinks.File` |

---

## 2. Project Structure

```
VideoCrop/
├── VideoCrop.sln
├── src/
│   ├── VideoCrop.Core/            # Pure .NET class library, no UI deps
│   │   ├── Processes/             # FfmpegRunner, FfprobeRunner, MpvHost
│   │   ├── Models/                # VideoInfo, EncodePreset, CutSpec, ResizeSpec, CropSpec
│   │   ├── Encoding/              # Codec definitions, preset library, command builders
│   │   ├── IO/                    # OutputNamer, ToolLocator, TempFileManager
│   │   └── Serialization/         # Strongly-typed DTOs for ffprobe / mpv IPC
│   ├── VideoCrop.App/             # WinUI 3 app
│   │   ├── App.xaml
│   │   ├── MainWindow.xaml
│   │   ├── Views/                 # SidePanel, VideoPane, ProgressView
│   │   ├── ViewModels/            # MVVM via CommunityToolkit.Mvvm runtime classes (ObservableObject, RelayCommand)
│   │   ├── Controls/              # AspectLockedSizeInput, TimeMsInput, CropOverlay
│   │   ├── Interop/               # Win32 helpers, HWND management, mpv embedding
│   │   └── Assets/
│   └── VideoCrop.Tests/           # xUnit, focuses on Core
└── tools/                          # Bundled binaries (gitignored, fetched by script)
    ├── ffmpeg.exe
    ├── ffprobe.exe
    └── mpv.exe
```

`VideoCrop.Core` must remain UI-free so it can be unit-tested without spinning up XAML.

---

## 3. External Tool Strategy

### Bundling

Ship `ffmpeg.exe`, `ffprobe.exe`, and `mpv.exe` in a `tools/` subdirectory next to the app's `.exe`. Add a `fetch-tools.ps1` script that downloads pinned versions during first build / CI. Do **not** check binaries into git.

### Tool resolution

`ToolLocator` resolves each tool in this order:
1. `tools/` directory next to the app executable (default — bundled).
2. Path configured by the user in Settings.
3. `PATH` environment variable (fallback for power users).

If a required tool is missing on startup, show a setup screen explaining the problem and offering to download.

### Licensing reminder

- **mpv** is GPLv2+. Because we spawn it as a separate process (not linking to libmpv), our app code is not infected. We must include mpv's license and offer source on request when distributing the binary.
- **ffmpeg** can be GPL or LGPL depending on the build. Bundle a build with the license profile that matches our app's license intent. The BtbN Windows builds are convenient and clearly licensed.

---

## 4. Process Management Pattern

A shared `ExternalProcess` helper wraps `System.Diagnostics.Process` with:
- Async stdout/stderr streaming with line-buffered callbacks.
- Cancellation via `CancellationToken` (sends `q` to mpv, `SIGINT`-equivalent / kill to ffmpeg).
- Progress parsing hook.
- Proper cleanup in `IAsyncDisposable.DisposeAsync`.

ffmpeg progress: invoke with `-progress pipe:1 -nostats` so we get a clean key=value stream on stdout instead of having to scrape the human-readable stderr output.

---

## 5. Feature Specifications

### 5.1 Video Import

Two entry points, both funneling into the same `LoadVideoAsync(string path)` flow:
- **Drag-and-drop** anywhere on the main window. Use WinUI 3's `DragDrop` APIs on the root grid. Accept `DataPackageView.Contains(StandardDataFormats.StorageItems)`.
- **File → Open** dialog (`FileOpenPicker`).

On load, immediately run ffprobe and populate `VideoInfo`:

```
ffprobe -v error -print_format json -show_format -show_streams "<input>"
```

Parse into a strongly-typed `VideoInfo` record (duration, width, height, sar, dar, codec, fps, bitrate, audio streams) using `JsonSerializer.Deserialize<FfprobeOutput>` with DTOs annotated via `[JsonPropertyName]`.

### 5.2 Compression

**Codecs supported:**

| User-facing | ffmpeg encoder      | Container default | Notes                                 |
|-------------|---------------------|-------------------|---------------------------------------|
| H.264       | `libx264`           | mp4               | Most compatible, fastest software enc |
| H.265       | `libx265`           | mp4               | ~40% smaller than H.264 at same CRF   |
| AV1         | `libsvtav1`         | mp4               | Use SVT-AV1, not libaom (10x faster)  |
| VP9         | `libvpx-vp9`        | webm              | Two-pass strongly recommended         |

All encoders are software (CPU). No hardware encoders (NVENC/AMF/QSV) — predictable behavior across machines, no GPU dependency, quality is consistently better at the same bitrate.

**Presets (the basic-mode dropdown):**

Presets target reasonable file sizes for typical web sharing — quality, not budget, drives the output. A 10-second clip has no reason to be 50 MB just because Discord allows it.

Presets are split into two groups so the tradeoff is obvious:

- **Compatibility** — H.264 everywhere. Plays in every browser, every chat app, every device. Pick these when in doubt.
- **Optimization** — H.265 or AV1. Smaller files at the same visual quality, but with caveats. Each shows a compatibility warning inline.

**Compatibility presets (H.264):**

| Preset           | Codec | CRF | Speed preset | Audio    | Description shown in UI                                            |
|------------------|-------|-----|--------------|----------|--------------------------------------------------------------------|
| **Web: High**    | H.264 | 19  | slow         | AAC 192k | Near-lossless. Best for archiving or re-editing.                   |
| **Web: Medium**  | H.264 | 23  | medium       | AAC 128k | Default. Standard web quality; small files, plays everywhere.      |
| **Web: Low**     | H.264 | 27  | medium       | AAC 96k  | Aggressive compression. Use for chat clips where size matters most. |

**Optimization presets:**

"Fast" = H.265 (encodes faster, modest compatibility caveats). "Slow" = AV1 (slower to encode, best compression, larger compatibility caveats).

| Preset                | Codec   | CRF | Speed preset | Audio    | Description shown in UI                                                                |
|-----------------------|---------|-----|--------------|----------|----------------------------------------------------------------------------------------|
| **High (Fast)**       | H.265   | 23  | slow         | AAC 192k | ~40% smaller than H.264 High at similar quality. ⚠️ May not play in Discord embeds, older browsers, or older devices. |
| **High (Slow)**       | SVT-AV1 | 28  | preset 5     | AAC 192k | Best compression at this quality tier. ⚠️ Limited compatibility (Safari, many chat apps, older hardware). Slower to encode. |
| **Medium (Fast)**     | H.265   | 27  | medium       | AAC 128k | Smaller than Web: Medium with similar visual quality. ⚠️ Same H.265 caveats as above.    |
| **Medium (Slow)**     | SVT-AV1 | 32  | preset 6     | AAC 128k | Smallest file at default quality. ⚠️ Same AV1 caveats as above.                          |
| **Low (Fast)**        | H.265   | 30  | medium       | AAC 96k  | Tiny chat-clip files. ⚠️ Same H.265 caveats as above.                                    |
| **Low (Slow)**        | SVT-AV1 | 36  | preset 7     | AAC 96k  | Smallest possible at watchable quality. ⚠️ Same AV1 caveats as above.                    |

**Custom** — opens the advanced panel with everything exposed. VP9 lives here (not in the preset list — niche enough that exposing it as a top-level preset would mislead users about its compatibility story).

The compatibility-warning text on Optimization presets must render inline in the preset description area when selected — not buried in a tooltip — so users understand the tradeoff before they hit Process.

**Advanced options panel (Custom preset, or "Show advanced" toggle):**

- Codec: H.264 | H.265 | AV1 | VP9
- Rate control mode: CRF | CBR | VBR (target bitrate) | Two-pass target size
- CRF value (slider 0–51 for x264/x265, 0–63 for AV1/VP9; show codec-appropriate range)
- Target bitrate (kbps/Mbps switchable input) — for when you genuinely need a specific size
- Target file size in MB (computes two-pass bitrate from duration) — for when you genuinely need to hit a hard cap
- Speed preset: ultrafast | superfast | veryfast | faster | fast | medium | slow | slower | veryslow (mapped per codec)
- Audio codec: AAC | Opus | MP3 | Copy
- Audio bitrate: 64 / 96 / 128 / 160 / 192 / 256 / 320 kbps
- Container override
- Pixel format: yuv420p (default) | yuv420p10le (10-bit, x265/AV1)

Target-size mode in the advanced panel computes video bitrate from `(target_bytes * 8 - audio_bitrate * duration) / duration` with a 3% safety margin, then encodes two-pass. This is the *only* place size-targeting lives — out of the main preset list intentionally.

The `EncodeCommandBuilder` takes a `CompressionSpec` record and produces a `List<string>` of ffmpeg args. Keep this pure and unit-test it heavily — bad command construction is the #1 source of bugs in apps like this.

### 5.3 Cutting (temporal trim)

UI:
- Two `TimeMsInput` controls (start, end), each parsing `HH:MM:SS.fff` and accepting direct numeric input down to milliseconds.
- "Set from playhead" button next to each (queries mpv's `time-pos` property via IPC).
- A timeline scrubber under the video with two draggable markers (one for start, one for end).
- Duration display showing `end - start`.

**Playback is constrained to the cut range.** As soon as either start or end is set, mpv playback can only move within `[start, end]`. The user sees exactly what their final cut will look like, scrub-able, loopable, with no way to wander outside the bounds. Implementation:

- On any change to start or end, push the values to mpv via IPC: `set_property ab-loop-a <start>` and `set_property ab-loop-b <end>`. mpv's A-B loop natively handles "jump back to A when reaching B," giving us free preview looping.
- All seek commands sent from our UI go through a clamp: `seekTime = Math.Clamp(requested, start, end)`. Belt-and-suspenders for cases the user expresses via the scrubber rather than playback.
- Subscribe to `time-pos` via `observe_property`. If a property update reports a time outside `[start, end]` (e.g., due to a buffered seek), immediately send a corrective `seek` to the nearest bound.
- When the user clears the cut (both inputs reset to 0..duration), unset the loop: `set_property ab-loop-a no` and `set_property ab-loop-b no`.

**Timeline visualization:**
- The scrubber spans the full video duration so the user keeps context for where their cut sits in the overall timeline.
- The `[start, end]` region renders at full opacity in the accent color; outside the range is dimmed to ~30% opacity to make the cut obvious at a glance.
- The playhead is rendered only inside `[start, end]`. Clicking on the dimmed (outside) region snaps the playhead to the nearest in-bounds edge rather than seeking there.
- Dragging the start handle past the end handle (or vice versa) is blocked — minimum cut duration is 100ms.

**Time display:**
- Primary readout shows position relative to the cut: `00:03.452 / 00:12.180` (where 12.18s is `end - start`).
- Secondary, smaller readout shows absolute position in the source: `(at 01:23.452 of 04:18.220)`. Lets power users still see where they are in the original.

ffmpeg command for the final encode:
- **Fast path (stream copy, keyframe-snapped):** `-ss <start> -to <end> -i <input> -c copy <output>`. Use when the user accepts keyframe snapping; warn that cuts may not be frame-accurate.
- **Accurate path (re-encode):** `-i <input> -ss <start> -to <end> -c:v <codec> -c:a <codec> ...`. Slower but exact. Use when combined with compression, or when user toggles "Accurate cut".

Note the `-ss` placement matters: before `-i` is fast/inaccurate (seek), after `-i` is slow/accurate (decode-then-cut). The command builder must handle this correctly.

### 5.4 Cropping (spatial)

Crop is one of the headline features (the app's namesake). It operates on the source frame in source-pixel coordinates and runs *before* resize in the ffmpeg pipeline. See §5.6 for how crop and resize interact.

Workflow — pause-and-crop, per the architecture decided earlier (best fit for the spawn-mpv model):

- User enters crop mode → mpv pauses and writes a screenshot via the `screenshot-to-file <temp>.png` IPC command.
- The mpv region is hidden, the screenshot is shown in a XAML `Image` with a draggable rectangle overlay drawn on top.
- The crop rectangle has 8 handles (4 corners, 4 edges) + a center drag region.
- Snap-to-aspect options on a small toolbar above the canvas: Free | 16:9 | 4:3 | 1:1 | 9:16 | Original.
- On commit, store as `CropSpec { X, Y, W, H }` in **source-pixel coordinates**. Apply via ffmpeg `-vf crop=W:H:X:Y` in the final encode. For live preview, re-show mpv with the `video-crop` property set so the user sees the cropped result during playback.
- "Reset crop" button clears the `CropSpec` and re-enables the full frame.

The crop overlay control is a self-contained XAML `UserControl` (`CropOverlay` in the project structure) that takes a source-pixel-size `Size` and emits `CropSpec` changes. Keep it ignorant of resize state — the cleanest separation.

### 5.5 Resizing

UI:
- Two numeric inputs: Width and Height, with a lock icon between them.
- When locked (default), changing one updates the other based on the **post-crop** aspect ratio.
- When unlocked, both are free — show a warning if the ratio diverges significantly.
- Preset buttons: 2160p, 1440p, 1080p, 720p, 480p (height-based, width follows the post-crop aspect ratio).
- "Input" display showing what the resize stage actually receives:
  - No crop: `1920×1080 (16:9)` — the source.
  - Crop active: `1600×900 (16:9) — cropped from 1920×1080` — the effective input to the scaler.

Math: compute display aspect ratio from `width * sar_num / sar_den` over `height`. Most modern videos have SAR 1:1, but treat anamorphic content correctly. After crop is applied, the effective input is the crop's W and H (SAR forced to 1:1 by the crop filter).

ffmpeg: `-vf scale=W:H:flags=lanczos`, chained after crop if both are active. Use `-2` for either dimension to let ffmpeg compute it from the other (ensures even numbers required by most codecs):
- Locked, user set width=1280: `scale=1280:-2`
- Locked, user set height=720: `scale=-2:720`
- Unlocked: `scale=W:H` directly

If output dimensions are odd, ffmpeg will fail on yuv420p. Always round to even.

### 5.6 Crop + Resize interaction (pipeline contract)

Crop and resize are separate UI sections but share an explicit pipeline. The rules:

1. **Order is fixed: crop, then resize.** ffmpeg filter graph is always `crop=W:H:X:Y,scale=W2:H2:flags=lanczos`. There is no UI option to reverse this — it would have no useful interpretation.
2. **Crop is defined in source-pixel coordinates.** Independent of any resize setting. Toggling, changing, or removing resize never invalidates a crop.
3. **Resize sees the post-crop frame as its input.** All readouts, aspect-ratio math, lock behavior, and resolution presets in the Resize section use post-crop dimensions when crop is active.
4. **When crop changes and Resize is aspect-locked**, the resize values auto-adjust on the locked axis to maintain the post-crop aspect ratio, with a small toast: `Resize updated to 1280×720 to match new aspect.`
5. **When crop changes and Resize is unlocked and now mismatches**, do not auto-adjust. Show an inline `⚠ aspect mismatch — output will be stretched` indicator in the Resize section. The user can either re-lock, pick a preset, or accept the stretch.
6. **The Output section shows the full pipeline** so the user can verify before processing: `Output: 1280×720 H.264 — crop 1600×900 from 1920×1080, then scale to 1280×720.`
7. **Either feature can be used independently.** Disabling crop reverts resize's "Input" readout to the source. Disabling resize means crop output goes straight to the encoder at its native dimensions.

The `EncodeCommandBuilder` composes the `-vf` filter chain from optional `CropSpec` + optional `ResizeSpec`. Unit tests for the builder must cover: crop only, resize only, both, neither, crop with odd dimensions, resize with odd dimensions, crop changing aspect ratio with resize locked vs unlocked.

### 5.7 Playback (mpv integration)

Per the architecture decided earlier:
- Spawn `mpv.exe` as a separate process with `--wid=<HWND>` pointing at a native Win32 child window we create inside the video pane region.
- IPC via named pipe: `--input-ipc-server=\\.\pipe\videocrop-mpv-<guid>`.
- Required mpv flags: `--idle=yes --force-window=yes --no-osc --no-input-default-bindings --keep-open=yes --pause`.
- Implement `MpvIpcClient` with async `SendCommandAsync<T>(params object[] command)` and an event stream for property changes (subscribe via `observe_property`).
- Properties we care about: `time-pos`, `duration`, `pause`, `video-params/w`, `video-params/h`, `ab-loop-a`, `ab-loop-b`.
- `MpvIpcClient` exposes a `SeekClampedAsync(double time)` helper that clamps against the current cut bounds before issuing the `seek` command. All UI seeks go through this; no caller speaks raw `seek` directly.

HWND management:
- Create a native child window via `CreateWindowEx` with `WS_CHILD | WS_VISIBLE`, parented to the main window's HWND (`WindowNative.GetWindowHandle(this)`).
- Reposition it on every `SizeChanged` of its XAML placeholder using `SetWindowPos`.
- Track DPI changes — multiple monitors with different scales is the most common bug source here.

---

## 6. Side Panel UI

A single fixed-width side panel (roughly 360px) on the right side of the window, with the video pane filling the remaining space.

Panel sections (collapsible accordion or tabs — prefer accordion so users can see multiple at once). The order top-to-bottom mirrors the processing pipeline:

1. **Source** — filename, duration, resolution, codec. Read-only summary.
2. **Cut** — start/end time inputs, accurate/fast toggle.
3. **Crop** — enable toggle, "Edit crop" button (enters crop mode), current crop summary (`1600×900 from 1920×1080`), reset.
4. **Resize** — width/height inputs, lock toggle, preset buttons. Input readout reflects post-crop dimensions when crop is active (see §5.6).
5. **Compression** — preset dropdown, advanced toggle, all options below.
6. **Output** — destination folder picker, format selector, full pipeline summary (`Output: 1280×720 H.264 — crop 1600×900 from 1920×1080, then scale to 1280×720`), filename preview.
7. **Action bar (fixed at panel bottom)** — Big "Process" button, progress bar, cancel.

When processing, the panel disables inputs (except cancel) and the action bar shows live progress (frame, fps, speed, ETA) parsed from ffmpeg's `-progress` output.

---

## 7. Output File Naming

`OutputNamer.GetNextAvailable(string sourcePath, string outputDir, string extension)` logic:

1. Strip extension from source filename → `baseName`.
2. Try `{outputDir}/{baseName}_VideoCrop{extension}` first.
3. If exists, try `{baseName}_VideoCrop2{extension}`, `_VideoCrop3{extension}`, etc.
4. Return the first non-existing path.

Standardize on `_VideoCrop` (capital V, capital C) for consistency — your spec had both `_Videocrop` and `_VideoCropX`; flagging in case you want the lowercase variant.

Unit tests should cover: collision with `_VideoCrop`, collision with `_VideoCrop2`, source already ending in `_VideoCrop`, unicode filenames, very long paths near `MAX_PATH`.

---

## 8. Progress Tracking

ffmpeg invocation includes `-progress pipe:1 -nostats`. Output is line-based key=value:

```
frame=123
fps=58.2
out_time_us=4920000
speed=1.95x
progress=continue
```

Parse with a single compiled `Regex` (`RegexOptions.Compiled` on a static field) and emit `EncodeProgress` events:

```csharp
public sealed record EncodeProgress(
    long Frame,
    double Fps,
    TimeSpan OutTime,
    double Speed,
    bool IsFinished);
```

ViewModel computes percentage from `OutTime / totalDuration` and an ETA from `(totalDuration - OutTime) / speed`.

---

## 9. Implementation Phases

Each phase should be independently runnable. Build and run Release (`dotnet publish -r win-x64 -c Release --no-self-contained`) at the end of each phase to catch issues that don't show up in Debug. The .NET 10 Desktop Runtime must be installed on the target machine — handle this in the installer (Phase 11) and in the "missing runtime" first-run experience.

### Phase 1 — Skeleton
- Create solution, three projects (`VideoCrop.Core`, `VideoCrop.App`, `VideoCrop.Tests`).
- Bare WinUI 3 window that launches and shows "Hello".
- Configure `CommunityToolkit.Mvvm`, `System.Text.Json`, and `Serilog` packages.
- `fetch-tools.ps1` script and `ToolLocator`.
- Smoke test: app launches, finds tools, reports versions.

### Phase 2 — ffprobe & video metadata
- `FfprobeRunner` with `GetVideoInfoAsync`.
- `VideoInfo` record plus internal ffprobe DTOs annotated with `[JsonPropertyName]`, deserialized via `JsonSerializer.Deserialize<T>`.
- Unit tests with a few sample videos (commit small `.mp4` fixtures or generate them in test setup with ffmpeg).

### Phase 3 — Drag-and-drop and source display
- Accept file drops on main window.
- Side panel "Source" section displays metadata.
- File Open dialog as alternative entry.

### Phase 4 — mpv playback
- Native child HWND.
- Spawn mpv with `--wid` + IPC pipe.
- `MpvIpcClient` with command/response and property observation.
- Play/pause button, scrubber bound to `time-pos`.
- HWND repositioning on layout changes.

### Phase 5 — Compression (basic)
- `CompressionSpec` and `EncodeCommandBuilder`.
- All four codecs working with default settings.
- "Process" button runs encode, output saved with `_VideoCrop` suffix.
- Progress bar live-updating.

### Phase 6 — Cutting
- Time inputs, "set from playhead" buttons.
- Timeline scrubber with two draggable handles, full-duration context with dimmed out-of-range regions.
- Playback constraint via mpv `ab-loop-a`/`ab-loop-b` properties.
- `SeekClampedAsync` helper on `MpvIpcClient`; all UI seeks routed through it.
- Both fast (stream copy) and accurate (re-encode) ffmpeg paths.
- Integrates with compression spec when both are configured.

### Phase 7 — Cropping
- `CropSpec` stored in source-pixel coordinates.
- Pause-and-crop overlay with screenshot via mpv `screenshot-to-file`.
- `CropOverlay` user control: 8 handles + center drag + aspect snap toolbar (Free / 16:9 / 4:3 / 1:1 / 9:16 / Original).
- ffmpeg `-vf crop` integration in `EncodeCommandBuilder`.
- Live preview after commit via mpv's `video-crop` property.
- Reset button.

### Phase 8 — Resizing (with crop-aware behavior)
- Aspect-locked width/height inputs.
- Resolution presets (height-based).
- Implements the crop+resize pipeline contract (§5.6): post-crop dimensions drive the aspect lock and presets; auto-adjust on crop change when locked; mismatch warning when unlocked.
- Pipeline summary line in Output section.

### Phase 9 — Compression (advanced) & presets
- Compatibility presets (Web: High / Medium / Low — all H.264).
- Optimization presets (High/Medium/Low × Fast(H.265)/Slow(AV1) — six total).
- Advanced options panel with VP9 + target-size / target-bitrate modes.
- Two-pass encoding for target-size mode.

### Phase 10 — Polish
- Settings page (tool paths, default output dir, default preset).
- Error reporting (show ffmpeg stderr in a collapsible panel on failure).
- Recent files list.
- Keyboard shortcuts (Space = play/pause, I/O = set in/out, etc. — match mpv conventions).
- Logging to file.

### Phase 11 — Packaging
- Decide unpackaged vs MSIX.
- Code signing.
- Installer or zip distribution.
- Runtime dependency: detect missing .NET 10 Desktop Runtime on first launch and prompt to install (or have the installer chain it in).
- License/about screen including third-party notices.

---

## 10. Open Questions To Confirm Before Starting

1. **Filename suffix capitalization** — `_VideoCrop` (my standardization) or `_Videocrop` (as you wrote it once)?
2. **Audio handling on cut-only operations** — stream copy by default, or always re-encode?
3. **Batch processing** — is multi-file queue in scope for v1, or single-file-at-a-time?
4. **Output container override** — should the user be able to pick any container, or do we hard-tie containers to codecs (mp4 for h264/h265/av1, webm for vp9)?
5. **Subtitles and multiple audio tracks** — handle them (copy/select), or strip everything except first video + first audio?
6. **Target Windows version** — Windows 10 (1809+) or Windows 11 only? Affects WinUI 3 baseline.

---

## 11. Risks & Mitigations

| Risk                                              | Mitigation                                              |
|---------------------------------------------------|---------------------------------------------------------|
| WinUI 3 quirks (XAML bindings, packaged vs unpackaged differences) | Test Release builds after every phase, not just Debug |
| ffmpeg command construction bugs                  | Pure command builder + extensive unit tests             |
| mpv HWND embedding on multi-DPI setups            | Test on multi-monitor with mixed DPI from Phase 4       |
| Two-pass encoding temp file cleanup on cancel     | `IAsyncDisposable` + `TempFileManager` with finalizer   |
| Long-running processes orphaned on app crash      | Job objects (`CreateJobObject` + `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) so child procs die with parent |
| User's ffmpeg build lacks an encoder (libsvtav1)  | Detect at startup, gray out unsupported codecs          |

---

## 12. Things Not To Do

- Don't link libmpv directly — defeats the licensing-isolation rationale for spawning mpv.
- Don't put `Task.Run` everywhere just to "make it async" — process I/O is already async; CPU work happens in the child process.
- Don't try to parse ffmpeg's human-readable stderr for progress; use `-progress pipe:1`.
- Don't trust user-supplied paths without quoting — always pass arguments via `ArgumentList`, never build a single command string.
- Don't only test Debug builds — package and run the Release output before declaring a phase complete.
