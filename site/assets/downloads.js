// Shared by every page: finds the latest GitHub release, picks the right installer for this
// visitor (x64 or ARM64) and fills in the page. Same markup contract as the other Vexbyte sites.
//
//   <a data-download="primary">     best installer for this visitor (href)
//   <span data-download-label>      e.g. "Download for Windows"
//   <span data-download-detail>     e.g. "v3.0.0 · x64 · 2.0 MB"
//   <span data-version>             e.g. "v3.0.0"
//   <a data-download="<target id>"> one link per target (see TARGETS)
//
// Until a release is published every link points at the Releases page.
(function () {
  const REPO = 'Azmekk/VideoCrop';
  const RELEASES = `https://github.com/${REPO}/releases/latest`;

  const TARGETS = [
    { id: 'windows-x64', label: 'Windows', detail: 'x64', match: /-x64-setup\.exe$/ },
    { id: 'windows-arm64', label: 'Windows (ARM)', detail: 'ARM64', match: /-arm64-setup\.exe$/ },
    { id: 'portable-x64', label: 'Portable', detail: 'x64 · single exe', match: /-x64-portable\.exe$/ },
    { id: 'portable-arm64', label: 'Portable (ARM)', detail: 'ARM64 · single exe', match: /-arm64-portable\.exe$/ },
  ];

  async function isArm() {
    try {
      const hints = await navigator.userAgentData?.getHighEntropyValues(['architecture']);
      if (hints?.architecture) return hints.architecture === 'arm';
    } catch {}
    return /arm|aarch64/i.test(navigator.userAgent);
  }

  const mb = (bytes) => (bytes ? `${(bytes / 1048576).toFixed(1)} MB` : '');
  const setText = (sel, text) => document.querySelectorAll(sel).forEach((el) => { el.textContent = text; });

  async function load() {
    const [arm, release] = await Promise.all([
      isArm(),
      fetch(`https://api.github.com/repos/${REPO}/releases/latest`, { headers: { Accept: 'application/vnd.github+json' } })
        .then((r) => (r.ok ? r.json() : null))
        .catch(() => null),
    ]);
    const assets = release?.assets ?? [];
    const found = TARGETS.map((t) => ({ ...t, asset: assets.find((a) => t.match.test(a.name)) }));
    const primary = found.find((t) => t.id === (arm ? 'windows-arm64' : 'windows-x64'));
    const version = release?.tag_name ?? null;

    // Only trust the release when it carries VideoCrop 3 installers (older 2.x releases shipped zips),
    // otherwise keep the page's static fallback text.
    if (version && primary?.asset) setText('[data-version]', version);
    setText('[data-download-label]', 'Download for Windows');
    if (primary?.asset) {
      setText('[data-download-detail]', [version, primary.detail, mb(primary.asset.size)].filter(Boolean).join(' · '));
    }
    document.querySelectorAll('[data-download]').forEach((a) => {
      const id = a.getAttribute('data-download');
      const t = id === 'primary' ? primary : found.find((f) => f.id === id);
      a.setAttribute('href', t?.asset?.browser_download_url ?? RELEASES);
    });
    return { version, targets: found, primary };
  }

  window.VideoCrop = { downloads: load() };
})();
