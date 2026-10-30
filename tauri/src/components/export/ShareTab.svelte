<script lang="ts">
  import { exporter } from "../../lib/state/export.svelte";
  import { formatBytes } from "../../lib/util/format";
  import Icon from "../ui/Icon.svelte";
  import type { PlatformPreset } from "../../lib/types";

  const platforms = $derived(exporter.catalog?.platforms ?? []);

  const ICONS: Record<string, string> = {
    discord: "discord", youtube: "youtube", x: "x", twitter: "x", instagram: "instagram",
    vertical: "vertical", tiktok: "vertical", whatsapp: "whatsapp", email: "email", mail: "email",
  };

  function meta(p: PlatformPreset) {
    const parts: string[] = [];
    parts.push(p.maxSizeBytes ? `≤ ${formatBytes(p.maxSizeBytes)}` : "No size cap");
    if (p.maxWidth && p.maxHeight) parts.push(`${p.maxWidth}×${p.maxHeight}`);
    return parts.join(", ");
  }
</script>

{#if platforms.length}
  <div class="grid" role="radiogroup" aria-label="Destination">
    {#each platforms as p (p.id)}
      <button class="card" role="radio" aria-checked={exporter.platformId === p.id} onclick={() => exporter.selectPlatform(p.id)}
        data-tip={p.description}>
        <span class="ic ic-{ICONS[p.icon] ?? 'export'}"><Icon name={ICONS[p.icon] ?? "export"} size={18} /></span>
        <span class="txt">
          <span class="name">{p.name}</span>
          <span class="meta tnum">{meta(p)}</span>
        </span>
      </button>
    {/each}
  </div>
  {#if !exporter.platformId}
    <p class="note">Pick where the video is going. Size, resolution and quality are chosen to fit.</p>
  {/if}
{:else}
  <p class="note">Loading presets…</p>
{/if}

<style>
  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; }
  .card {
    display: flex; align-items: center; gap: 9px; padding: 8px 9px; text-align: left; min-width: 0;
    border-radius: var(--r-3); border: 1px solid var(--border); background: var(--surface);
    transition: border-color var(--fast), background var(--fast);
  }
  .card:hover { border-color: var(--border-strong); }
  .card[aria-checked="true"] { border-color: var(--accent); background: var(--accent-soft); box-shadow: inset 0 0 0 1px var(--accent); }
  .ic { flex: none; width: 30px; height: 30px; border-radius: var(--r-2); display: grid; place-items: center; background: var(--surface-2); color: var(--text-2); }
  .ic-discord { color: #5865f2; }
  .ic-youtube { color: #ff3b30; }
  .ic-whatsapp { color: #25d366; }
  .txt { display: flex; flex-direction: column; min-width: 0; }
  .name { font-weight: 600; font-size: 12.5px; line-height: 1.25; }
  .meta { font-size: 11px; color: var(--text-2); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
