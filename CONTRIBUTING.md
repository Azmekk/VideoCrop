# Contributing to VideoCrop

Thanks for helping out! Bug reports, ideas and pull requests are all welcome.

## Reporting bugs

Please [open an issue](https://github.com/Azmekk/VideoCrop/issues/new/choose) using the bug report template. The most useful details are:

- the VideoCrop version and Windows version,
- what kind of file it was (container, codec, resolution, length), and where it came from (OBS, phone, …),
- what you clicked, and what happened instead.

If an export failed, expand **ffmpeg output** on the error card and paste that too.

## Development setup

You need [Rust](https://rustup.rs/), Node 24 and [pnpm](https://pnpm.io/). An ffmpeg on your PATH saves the first-run download.

```bash
cd tauri
pnpm install
pnpm tauri dev
```

`pnpm dev` runs only the UI in a browser with a mock backend. Put a sample clip at `tauri/dev-media/sample.mp4` for it to play.

Read [`tauri/ARCHITECTURE.md`](tauri/ARCHITECTURE.md) before making larger changes. In short:

- `src-tauri/src/core/` is pure, deterministic logic (ffmpeg arguments, planner, presets) with unit tests. Keep processes and I/O out of it.
- `src-tauri/src/core/models.rs` and `src/lib/types.ts` are the IPC contract. Change them together.
- All frontend calls to Rust go through `src/lib/api.ts`.
- Small bundle and fast startup are product goals. Think twice before adding a dependency.

## Before opening a pull request

```bash
cd tauri
pnpm check
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
```

CI runs the same checks. If you change how ffmpeg commands are built (`core/encode.rs`, `core/preview.rs`, `core/planner.rs`), add or extend tests for it.
