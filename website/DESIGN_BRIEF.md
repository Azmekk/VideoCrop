# VideoCrop website — design brief (shared by all 5 concepts)

One static landing page per concept at `public/designs/<n>-<slug>/index.html`, hosted on Cloudflare
Pages (`wrangler.toml`, output dir `public/`). The owner will pick one and host it under a
`*.vexbyte.com` domain next to the sibling sites, later shown together in a "Vexbyte suite".

## The product
**VideoCrop 3**: a free Windows app to cut, crop, resize and compress videos. Rebuilt on Tauri.
- About 2 MB installer, about 5 MB app, and it opens instantly. Windows 10 and 11, x64 and ARM64.
- **Cut**: zoomable timeline with filmstrip; fine scrubbing (drag away from the track to slow down); typed times; frame-accurate or instant keyframe cuts.
- **Crop** on a full-resolution still; aspects 16:9, 4:3, 1:1, 9:16, 4:5, original; live crop preview.
- **Resize** with aspect lock, presets that never upscale, 75/50/25% options.
- **Share presets**, one click each: Discord (10 MB), Discord Nitro Basic (50 MB), Discord Nitro (500 MB), YouTube, X / Twitter (512 MB), TikTok / Reels / Shorts (vertical 1080×1920), WhatsApp (16 MB), Email (25 MB). Size caps are hit with two-pass encoding, and the resolution is picked automatically so the clip still looks good.
- **Quality presets** (H.264 / H.265 / AV1) and a Custom panel (CRF, bitrate, target size, 10-bit, audio codec). Can also export audio-only or video-only.
- **Plays everything**: multi-hour OBS recordings in HEVC, AV1 or MKV open instantly. Files Windows can't decode are streamed through an on-demand preview, using the GPU when available. The export is always made from the original.
- **Private**: everything runs locally with ffmpeg. There's no telemetry and no uploads. It uses your own ffmpeg if one is on your PATH; otherwise it downloads one once.
- Light and dark themes. A popup lets you know when a new version is out.
- Source on GitHub. **Do not** say "open source", "MIT" or name any license (none is chosen yet). "Free" is fine.

Links: GitHub `https://github.com/Azmekk/VideoCrop`, releases `https://github.com/Azmekk/VideoCrop/releases`,
issues `https://github.com/Azmekk/VideoCrop/issues`.

## Suite look (match the siblings)
Study https://storagemap.vexbyte.com/ and https://meetinghelper.vexbyte.com/ via the screenshots in
`C:\Users\marti\AppData\Local\Temp\claude\C--Users-marti-code-projects-VideoCrop\0b31b798-3541-4907-bfec-aa5115e3a74a\scratchpad\site-*.png`.
Both have:
- A near-black background (`#05060a` / `#050707`).
- A slim header: app icon + name on the left; nav links plus a pill Download/GitHub button on the right.
- A **centered hero**:
  - a small pill above the headline (status dot + "v3.0.0 is out", using `data-version`);
  - a huge display headline, with gradient text on key words;
  - a one-line subtitle;
  - a **big rounded download button** with an icon, the label "Download for Windows", and a mono detail line ("v3.0.0 · x64 · 2.0 MB");
  - under it, underlined secondary links ("Source on GitHub") and a muted line "Windows 10 and 11 · x64 & ARM64".
- An app-themed animated background behind the hero (storagemap: rotating sunburst rings; meetinghelper: a live audio waveform that reacts to the mouse).
- The product screenshot in a window frame below the hero.
- A row of 4 stat cards.
- Sections with a small uppercase **mono kicker** ("PICK A VIEW", "UNDER THE HOOD") above a large heading.
- Content that reveals on scroll.
- A closing CTA that repeats the download button.
- A footer: "© 2026 Vexbyte" on the left; links (Changelog, Source, Releases, Report an issue) on the right.

Typography: each sibling has its own display font, a clean body font, and JetBrains Mono for details.
Pick your own distinctive **display** font, but not Bricolage Grotesque or Unbounded (taken by the
siblings). Load fonts from Google Fonts only.

VideoCrop identity: amber accent `#f5b83d` (hover `#ffc857`, dark ink on amber `#1b1405`), app tile
greys `#16181d`→`#2a2e36`, light frame colour `#e7e9ee`. The icon is a rounded frame with amber
crop brackets.

## Assets (relative to a design page: `../../assets/`)
- `icon.svg`: the app icon (use it as the favicon and the header logo).
- `editor-dark.webp` (1440×900): the editor in dark theme, with the timeline, a cut and the Discord preset.
- `crop-light.webp` (1440×900): crop mode in light theme.
- `downloads.js`: include it with `<script src="../../assets/downloads.js" defer></script>`. It fills these hooks:
  - `<a data-download="primary">`: the right installer for the visitor;
  - `<span data-download-label>`, `<span data-download-detail>` and `<span data-version>`;
  - per-target links `data-download="windows-x64|windows-arm64|portable-x64|portable-arm64"`.

  Put sensible static fallback text in each element ("v3.0.0", "v3.0.0 · x64 · 2.0 MB"). Without a release, every link points at the releases page.

## Technical rules
- One self-contained `index.html` per design (inline CSS + JS). No frameworks and no JS libraries or CDNs, apart from Google Fonts. Vanilla JS, canvas/SVG/CSS animation.
- Animations should be smooth, prefer transform/opacity and canvas, and pause offscreen (IntersectionObserver / visibilitychange). Honour `prefers-reduced-motion` with a calm static fallback.
- Responsive down to 375 px wide, with no horizontal scroll. Use semantic HTML, give images alt text, use real buttons and links, and show visible focus states. Include `<title>`, a meta description, Open Graph tags, and `<meta name="theme-color">`.
- Reasonable weight: keep the page under ~60 KB of HTML/CSS/JS, apart from the shared assets.
