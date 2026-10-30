<script lang="ts" module>
  // 24×24 stroke icons (1.6px). Filled glyphs are prefixed with "F:".
  const P: Record<string, string> = {
    play: "F:M8 5.5v13a.6.6 0 0 0 .9.5l10.2-6.5a.6.6 0 0 0 0-1L8.9 5a.6.6 0 0 0-.9.5z",
    pause: "F:M7 5h3.2v14H7zM13.8 5H17v14h-3.2z",
    volume: "M4 9.5h3.5L12 6v12l-4.5-3.5H4zM15.5 9a4 4 0 0 1 0 6M18 6.5a7.5 7.5 0 0 1 0 11",
    mute: "M4 9.5h3.5L12 6v12l-4.5-3.5H4zM16 9.5l5 5M21 9.5l-5 5",
    folder: "M3.5 7.5a2 2 0 0 1 2-2h4l2 2h7a2 2 0 0 1 2 2v7.5a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z",
    file: "M6 3.5h8l4 4V20a.5.5 0 0 1-.5.5h-11A.5.5 0 0 1 6 20zM14 3.5V8h4",
    film: "M4 4.5h16v15H4zM8 4.5v15M16 4.5v15M4 9h4M4 15h4M16 9h4M16 15h4",
    crop: "M6.5 3v13.5H21M3 6.5h13.5V21",
    cut: "M6 8.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5zM6 20.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5zM8 7.5 20 17M8 16.5 20 7",
    resize: "M14 4h6v6M10 20H4v-6M20 4l-7 7M4 20l7-7",
    export: "M12 3.5v11M7.5 8 12 3.5 16.5 8M4.5 14.5V19a1.5 1.5 0 0 0 1.5 1.5h12a1.5 1.5 0 0 0 1.5-1.5v-4.5",
    close: "M6 6l12 12M18 6 6 18",
    min: "M5 12h14",
    max: "M5.5 5.5h13v13h-13z",
    restore: "M8 8h11v11H8zM5 16V5h11",
    sun: "M12 16.5a4.5 4.5 0 1 0 0-9 4.5 4.5 0 0 0 0 9zM12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M5.3 18.7l1.4-1.4M17.3 6.7l1.4-1.4",
    moon: "M19.5 14.5A8 8 0 0 1 9.5 4.5a8 8 0 1 0 10 10z",
    system: "M3.5 5h17v11h-17zM9 20h6M12 16v4",
    lock: "M6.5 11h11v9h-11zM8.5 11V8a3.5 3.5 0 0 1 7 0v3",
    unlock: "M6.5 11h11v9h-11zM8.5 11V8a3.5 3.5 0 0 1 6.8-1.2",
    check: "M5 12.5l4.5 4.5L19 7.5",
    alert: "M12 4 21 19.5H3zM12 10v4.5M12 17v.5",
    info: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 11v5.5M12 7.5v.5",
    chevron: "M7 10l5 5 5-5",
    zoomIn: "M10.5 17.5a7 7 0 1 0 0-14 7 7 0 0 0 0 14zM20.5 20.5l-5-5M7.5 10.5h6M10.5 7.5v6",
    zoomOut: "M10.5 17.5a7 7 0 1 0 0-14 7 7 0 0 0 0 14zM20.5 20.5l-5-5M7.5 10.5h6",
    fitCut: "M4 7V4h3M20 7V4h-3M4 17v3h3M20 17v3h-3M9 9v6M15 9v6M9 12h6",
    fitAll: "M3.5 12h17M6.5 9l-3 3 3 3M17.5 9l3 3-3 3",
    markIn: "M9 4H6v16h3M12 8l4 4-4 4",
    markOut: "M15 4h3v16h-3M12 8l-4 4 4 4",
    frameBack: "M16 6l-6 6 6 6M7 6v12",
    frameFwd: "M8 6l6 6-6 6M17 6v12",
    toStart: "M6 5v14M18 6l-8.5 6L18 18z",
    toEnd: "M18 5v14M6 6l8.5 6L6 18z",
    reveal: "M3.5 7.5a2 2 0 0 1 2-2h4l2 2h7a2 2 0 0 1 2 2V11M3.5 7.5V17a2 2 0 0 0 2 2H11M15 15h6M18 12v6",
    refresh: "M19.5 12a7.5 7.5 0 1 1-2.2-5.3M19.5 4.5v4h-4",
    download: "M12 3.5v11M7.5 10 12 14.5 16.5 10M4.5 18.5h15",
    eye: "M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12zM12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
    reset: "M4.5 12a7.5 7.5 0 1 0 2.2-5.3M4.5 4.5v4h4",
    stream: "M4 18a14 14 0 0 1 0-12M8 15.5a8 8 0 0 1 0-7M20 18a14 14 0 0 0 0-12M16 15.5a8 8 0 0 0 0-7M12 13.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z",
    gauge: "M4.5 17a8.5 8.5 0 1 1 15 0M12 13l4-5",
    plus: "M12 5v14M5 12h14",
    // Platform glyphs (simplified silhouettes).
    discord: "F:M18.6 6.3A15 15 0 0 0 15 5.2l-.5.9a14 14 0 0 0-5 0L9 5.2a15 15 0 0 0-3.6 1.1C3.1 9.8 2.5 13.2 2.8 16.5a15 15 0 0 0 4.5 2.3l1-1.6a9.6 9.6 0 0 1-1.6-.8l.4-.3a10.7 10.7 0 0 0 9.8 0l.4.3a9.6 9.6 0 0 1-1.6.8l1 1.6a15 15 0 0 0 4.5-2.3c.4-3.8-.6-7.2-2.6-10.2zM9.2 14.5c-.9 0-1.6-.8-1.6-1.8s.7-1.8 1.6-1.8 1.6.8 1.6 1.8-.7 1.8-1.6 1.8zm5.6 0c-.9 0-1.6-.8-1.6-1.8s.7-1.8 1.6-1.8 1.6.8 1.6 1.8-.7 1.8-1.6 1.8z",
    youtube: "F:M21.6 7.2a2.5 2.5 0 0 0-1.8-1.8C18.2 5 12 5 12 5s-6.2 0-7.8.4A2.5 2.5 0 0 0 2.4 7.2 26 26 0 0 0 2 12a26 26 0 0 0 .4 4.8 2.5 2.5 0 0 0 1.8 1.8C5.8 19 12 19 12 19s6.2 0 7.8-.4a2.5 2.5 0 0 0 1.8-1.8A26 26 0 0 0 22 12a26 26 0 0 0-.4-4.8zM10 15V9l5.2 3z",
    x: "F:M17.2 4h2.9l-6.3 7.2L21.2 20h-5.8l-4.5-5.9L5.7 20H2.8l6.7-7.7L2.4 4h5.9l4.1 5.4zm-1 14.3h1.6L7.9 5.6H6.2z",
    vertical: "M8 2.5h8a1.5 1.5 0 0 1 1.5 1.5v16a1.5 1.5 0 0 1-1.5 1.5H8A1.5 1.5 0 0 1 6.5 20V4A1.5 1.5 0 0 1 8 2.5zM11 18.5h2M10.5 9.5v4l3.5-2z",
    instagram: "M7.5 3.5h9a4 4 0 0 1 4 4v9a4 4 0 0 1-4 4h-9a4 4 0 0 1-4-4v-9a4 4 0 0 1 4-4zM12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7zM17 7v.1",
    whatsapp: "M4 20l1.2-4.1A8.5 8.5 0 1 1 8.4 19zM9 8.5c0 3.2 3.3 6.5 6.5 6.5l1-1.5-2-1-1 .8a5 5 0 0 1-2.3-2.3l.8-1-1-2z",
    email: "M3.5 6h17v12h-17zM3.5 6.5 12 13l8.5-6.5",
  };
</script>

<script lang="ts">
  let { name, size = 16, class: cls = "" }: { name: string; size?: number; class?: string } = $props();
  const d = $derived(P[name] ?? P.info);
  const filled = $derived(d.startsWith("F:"));
</script>

<svg
  class={cls}
  width={size}
  height={size}
  viewBox="0 0 24 24"
  aria-hidden="true"
  fill={filled ? "currentColor" : "none"}
  stroke={filled ? "none" : "currentColor"}
  stroke-width="1.6"
  stroke-linecap="round"
  stroke-linejoin="round"
><path d={filled ? d.slice(2) : d} /></svg>
