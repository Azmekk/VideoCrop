# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Windows desktop app for compressing, cutting, cropping, and resizing videos using ffmpeg (encode), ffprobe (metadata), and mpv (playback). The design document at `VideoCrop_Plan.md` is the source of truth for behavior — consult it before adding features.

## Commands

The solution file is `VideoCrop.slnx` (not `.sln`).

- **Build:** `dotnet build VideoCrop.slnx -c Debug`
- **Run all tests:** `dotnet test VideoCrop.slnx -c Debug`
- **Run a single test class:** `dotnet test VideoCrop.slnx --filter "FullyQualifiedName~EncodeCommandBuilderTests"`
- **Run a single test:** `dotnet test VideoCrop.slnx --filter "FullyQualifiedName~OutputNamerTests.Collision_with_base_uses_index_2"`
- **Run the app:** `dotnet run --project src/VideoCrop.App` (the WinUI 3 unpackaged-launch path through `Microsoft.Windows.SDK.BuildTools.WinApp`)
- **Release publish:** `dotnet publish src/VideoCrop.App -r win-x64 -c Release --no-self-contained`
- **External tools**: ffmpeg/ffprobe (BtbN GPL Windows build) and mpv (shinchiro Windows build) are downloaded into `tools/` next to the exe by `ToolDownloadService` at runtime, the first time the app launches with the tools missing. There is no separate fetch script — the workflow ships only .NET binaries; the app pulls the tools.

Targets: `net10.0-windows10.0.26100.0` (App) and `net10.0` (Core/Tests). `TargetPlatformMinVersion` is `10.0.17763.0` (Win10 1809+).

## Architecture

The solution is intentionally split so the encode/playback logic stays UI-free and unit-testable.

- **`VideoCrop.Core`** — pure .NET library, no UI deps. Contains models (`CompressionSpec`, `CutSpec`, `CropSpec`, `ResizeSpec`, `EncodeJob`, `VideoInfo`), process wrappers (`ExternalProcess`, `FfprobeRunner`, `FfmpegRunner`, `MpvHost`, `MpvIpcClient`), IO helpers (`ToolLocator`, `OutputNamer`, `TempFileManager`), and the encode command construction (`EncodeCommandBuilder`, `EncodeProgressParser`, `PresetLibrary`).
- **`VideoCrop.App`** — WinUI 3 (Windows App SDK 2.0+). MVVM is wired with **`CommunityToolkit.Mvvm` runtime classes only** (`ObservableObject`, `RelayCommand`) — no source-generator attributes. JSON uses `System.Text.Json` reflection-based deserialization with `[JsonPropertyName]` DTOs.
- **`VideoCrop.Tests`** — xUnit + FluentAssertions. Has `InternalsVisibleTo` access to Core (via `src/VideoCrop.Core/Properties/AssemblyInfo.cs`) so testable helpers can stay `internal` (e.g. `FfprobeRunner.ParseVideoInfo`).

### External-tool resolution

All ffmpeg/ffprobe/mpv invocations go through `IToolLocator`, which resolves in order: bundled `tools/` next to the app exe → user-configured path in settings → `PATH`. If you add a new external-tool call, route it through `ToolLocator.TryResolve` — don't shell out directly.

### mpv embedding

- The app spawns mpv as a separate process (not linking to libmpv) for GPLv2+ isolation. Pass `--wid=<HWND>` pointing at a native Win32 child window created by `VideoCrop.App/Interop/VideoHostWindow.cs` (via `CreateWindowExW`) parented to the WinUI window's HWND.
- The native window is repositioned on every `SizeChanged` of the XAML placeholder via `SetWindowPos`, scaled by `GetDpiForWindow`. Multi-monitor mixed-DPI is the most common breakage.
- IPC is over a named pipe (`\\.\pipe\videocrop-mpv-<guid>`) using `MpvIpcClient` — a `NamedPipeClientStream` with an async read loop that demultiplexes property changes (`observe_property`) from command replies (matched by `request_id`).
- `PlayerViewModel.SeekClampedAsync` / `MpvIpcClient.SeekClampedAsync` enforces playback bounds when a cut is active. Bounds also flow to mpv via `ab-loop-a` / `ab-loop-b` for free preview looping. UI code should not call raw `seek` — always go through the clamped helper.

### ffmpeg progress

Encoder invocations include `-progress pipe:1 -nostats`. Parse the resulting line-buffered key=value stream with `EncodeProgressParser` (a stateful per-encode instance). Do not scrape the human-readable stderr for progress.

### Crop + Resize pipeline contract

`EncodeCommandBuilder` always composes the filter chain as `crop=W:H:X:Y,scale=W2:H2:flags=lanczos` — fixed order. Rules enforced across the code:

- `CropSpec` is always in **source-pixel coordinates**, independent of resize.
- Resize sees the **post-crop** frame as its input. Aspect-lock math, height presets, and the "Input" readout in `ResizeViewModel` all use post-crop dimensions when crop is active.
- Output dimensions must be even (yuv420p constraint). `ResizeSpec.WithEvenDimensions()` rounds down; the command builder uses `-2` for the unlocked axis when aspect-locked.
- The pipeline summary (`MainViewModel.PipelineSummary`) is the single source of truth shown to the user before they hit Process — keep it accurate when changing how crop or resize interact.

When changing `EncodeCommandBuilder`, also extend `tests/VideoCrop.Tests/EncodeCommandBuilderTests.cs`. Bad command construction is the #1 source of bugs in apps like this — the builder is a pure function specifically so it can be covered exhaustively.

### Output filenames

`OutputNamer.GetNextAvailable` standardizes on suffix `_VideoCrop` (capital V, capital C). It strips an existing `_VideoCrop[N]` suffix from the source basename so re-encoding an already-suffixed file doesn't double up.

## Conventions

- **Primary constructors** are used for services with a simple inject-and-store pattern (`FfprobeRunner`, `FfmpegRunner`, `MpvHost`, `MpvIpcClient`, `ToolLocator`, most ViewModels, `AppServices`, `ExternalProcessException`). Constructors that wire events or build child VMs (e.g. `MainViewModel`, `CropDialog`) keep a classic body.
- **XAML namespace prefixes** must all be declared. Using a `views:` or `ctrl:` prefix without the matching `xmlns:` declaration causes the WinUI XAML compiler to exit with code 1 and **no `output.json` / no human-readable error** — only the symptom that pass 2 fails silently. If you see `MSB3073` from `XamlCompiler.exe` with no other diagnostic, that's the first thing to check.
- **No `Task.Run` for I/O.** Process I/O is already async via `Process.WaitForExitAsync` and stream events. Don't wrap async-over-sync.
- **Arguments via `ArgumentList`**, never a single command string — see `ExternalProcessOptions` and how `EncodeCommandBuilder` emits `List<string>`. Path quoting is delegated to `ProcessStartInfo`.
- **Sample-aspect-ratio awareness:** `VideoInfo.DisplayAspectRatio` accounts for anamorphic SAR. Don't assume SAR 1:1.
- **Cut path selection:** `-ss` before `-i` is fast/inaccurate (keyframe-snapped seek); after `-i` is slow/accurate (decode-then-cut). `EncodeCommandBuilder` flips the placement based on `CutSpec.Accurate`.

## Key open items

The plan's §10 "Open Questions" got these defaults during initial implementation: `_VideoCrop` suffix capitalization, stream-copy audio on cut-only, single-file processing in v1, container hard-tied to codec with override in advanced panel, strip extras to first video + first audio, Win10 1809+ baseline. Confirm with the user before changing any of these.

Packaging (Phase 11): MSIX vs unpackaged distribution, code signing, installer, and .NET 10 Desktop Runtime detection on first launch are all deferred — `dotnet publish` works but nothing is wrapped or signed yet.
