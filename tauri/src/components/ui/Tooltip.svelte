<script lang="ts">
  // One global tooltip for every element with `data-tip`. Fixed-positioned so scroll
  // containers never clip it.
  import { onMount } from "svelte";

  let text = $state("");
  let x = $state(0);
  let y = $state(0);
  let below = $state(false);
  let visible = $state(false);
  let el = $state<HTMLDivElement>();
  let timer: ReturnType<typeof setTimeout> | undefined;
  let target: HTMLElement | null = null;

  function place() {
    if (!target || !el) return;
    const r = target.getBoundingClientRect();
    const w = el.offsetWidth, h = el.offsetHeight;
    below = r.top - h - 8 < 40;
    x = Math.min(window.innerWidth - w - 8, Math.max(8, r.left + r.width / 2 - w / 2));
    y = below ? r.bottom + 6 : r.top - h - 6;
  }

  onMount(() => {
    const over = (e: PointerEvent) => {
      const t = (e.target as HTMLElement | null)?.closest?.("[data-tip]") as HTMLElement | null;
      if (t === target) return;
      target = t;
      clearTimeout(timer);
      visible = false;
      if (!t) return;
      timer = setTimeout(() => {
        if (target !== t || !t.dataset.tip) return;
        text = t.dataset.tip;
        requestAnimationFrame(() => { place(); visible = true; });
      }, 450);
    };
    const hide = () => { clearTimeout(timer); visible = false; target = null; };
    document.addEventListener("pointerover", over);
    document.addEventListener("pointerdown", hide, true);
    document.addEventListener("wheel", hide, { passive: true, capture: true });
    return () => {
      document.removeEventListener("pointerover", over);
      document.removeEventListener("pointerdown", hide, true);
      document.removeEventListener("wheel", hide, true);
    };
  });
</script>

<div class="tip" class:visible class:below bind:this={el} style:left="{x}px" style:top="{y}px" role="tooltip">{text}</div>

<style>
  .tip {
    position: fixed; z-index: 1000; max-width: 280px; pointer-events: none;
    padding: 6px 9px; border-radius: var(--r-2); font-size: 12px; line-height: 1.4;
    background: var(--surface-3); color: var(--text); border: 1px solid var(--border-strong);
    box-shadow: var(--shadow-2);
    opacity: 0; transform: translateY(2px); transition: opacity var(--fast), transform var(--fast);
  }
  .tip.below { transform: translateY(-2px); }
  .tip.visible { opacity: 1; transform: none; }
</style>
