<script lang="ts">
  import { chooseAndOpen, openVideo } from "../lib/state/pipeline.svelte";
  import { editor } from "../lib/state/editor.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { basename, dirname } from "../lib/util/format";
  import Icon from "./ui/Icon.svelte";

  let { dragOver = false }: { dragOver?: boolean } = $props();

  const recent = $derived(settings.value.recentFiles.slice(0, 6));
</script>

<section class="empty">
  <button class="zone" class:over={dragOver} class:busy={editor.opening} onclick={chooseAndOpen} disabled={editor.opening}>
    <span class="frame" aria-hidden="true">
      <span class="c tl"></span><span class="c tr"></span><span class="c bl"></span><span class="c br"></span>
      <Icon name={editor.opening ? "film" : "plus"} size={26} />
    </span>
    <span class="title">{editor.opening ? "Reading video…" : dragOver ? "Drop to open" : "Drop a video here"}</span>
    <span class="sub">or click to choose a file <kbd>Ctrl</kbd><kbd>O</kbd></span>
  </button>

  {#if editor.openError}
    <div class="error" role="alert">
      <Icon name="alert" size={15} />
      <div><strong>Couldn't open that file.</strong> {editor.openError}</div>
    </div>
  {/if}

  {#if recent.length}
    <div class="recent">
      <h2 class="label">Recent</h2>
      <ul>
        {#each recent as p (p)}
          <li>
            <button onclick={() => openVideo(p)} disabled={editor.opening} title={p}>
              <Icon name="film" size={15} />
              <span class="rn">{basename(p)}</span>
              <span class="rd">{dirname(p)}</span>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</section>

<style>
  .empty {
    flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 20px; padding: 32px; overflow: auto;
  }
  .zone {
    width: min(560px, 100%); aspect-ratio: 16 / 8; display: flex; flex-direction: column;
    align-items: center; justify-content: center; gap: 6px; border-radius: var(--r-4);
    border: 1px dashed var(--border-strong); background: var(--panel); color: var(--text);
    transition: border-color var(--med), background var(--med);
  }
  .zone:hover:not(:disabled), .zone.over { border-color: var(--accent-line); background: var(--surface); }
  .zone.over { border-style: solid; border-color: var(--accent); }
  .frame {
    position: relative; width: 72px; height: 52px; display: grid; place-items: center;
    color: var(--text-2); margin-bottom: 12px; transition: color var(--med), transform var(--med) var(--ease);
  }
  .zone:hover .frame, .zone.over .frame { color: var(--accent-text); }
  .zone.over .frame { transform: scale(1.06); }
  .c { position: absolute; width: 14px; height: 14px; border: 2px solid var(--accent); }
  .tl { top: 0; left: 0; border-right: 0; border-bottom: 0; border-top-left-radius: 4px; }
  .tr { top: 0; right: 0; border-left: 0; border-bottom: 0; border-top-right-radius: 4px; }
  .bl { bottom: 0; left: 0; border-right: 0; border-top: 0; border-bottom-left-radius: 4px; }
  .br { bottom: 0; right: 0; border-left: 0; border-top: 0; border-bottom-right-radius: 4px; }
  .busy .frame { animation: pulse 1.2s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: .4; } }
  .title { font: 600 18px/1.2 var(--font-display); }
  .sub { color: var(--text-2); display: flex; align-items: center; gap: 4px; }
  kbd {
    font: 500 11px/1 var(--font); padding: 3px 5px; border-radius: var(--r-1);
    border: 1px solid var(--border); border-bottom-width: 2px; background: var(--surface); color: var(--text-2);
  }
  .error {
    width: min(560px, 100%); display: flex; gap: 10px; padding: 10px 12px; border-radius: var(--r-3);
    background: var(--danger-soft); color: var(--text); user-select: text;
  }
  .error :global(svg) { color: var(--danger); flex: none; margin-top: 2px; }
  .recent { width: min(560px, 100%); }
  h2 { margin: 0 0 6px 2px; font-weight: 500; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 1px; }
  li button {
    width: 100%; display: flex; align-items: center; gap: 10px; padding: 7px 10px; border: 0;
    border-radius: var(--r-2); background: transparent; text-align: left; color: var(--text);
  }
  li button:hover:not(:disabled) { background: var(--surface-2); }
  li button :global(svg) { color: var(--text-3); flex: none; }
  .rn { white-space: nowrap; }
  .rd { color: var(--text-3); font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
</style>
