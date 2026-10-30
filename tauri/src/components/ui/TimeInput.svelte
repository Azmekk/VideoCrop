<script lang="ts">
  // Accepts H:MM:SS.mmm, MM:SS.mmm or plain seconds. ↑/↓ or the wheel step the segment under
  // the caret (Shift = ×10). Invalid input reverts on commit.
  import { formatTime, parseTime, segmentStep } from "../../lib/util/time";

  let { value, onchange, label, withHours = false, disabled = false }: {
    value: number; onchange: (v: number) => void; label: string; withHours?: boolean; disabled?: boolean;
  } = $props();

  let input = $state<HTMLInputElement>();
  let text = $state("");
  let editing = $state(false);
  let invalid = $state(false);

  $effect(() => {
    const f = formatTime(value, withHours);
    if (!editing) text = f;
  });

  function commit() {
    const v = parseTime(text);
    if (v == null) {
      invalid = true;
      setTimeout(() => (invalid = false), 600);
      text = formatTime(value, withHours);
      return;
    }
    if (Math.abs(v - value) >= 0.0005) onchange(v);
    // The store may clamp; reflect what was actually applied.
    queueMicrotask(() => (text = formatTime(value, withHours)));
  }

  function step(dir: number, big: boolean) {
    if (!input) return;
    const caret = input.selectionStart ?? text.length;
    const cur = parseTime(text) ?? value;
    onchange(Math.max(0, cur + dir * segmentStep(text, caret) * (big ? 10 : 1)));
    queueMicrotask(() => {
      text = formatTime(value, withHours);
      // Keep the caret in the same segment (formats are fixed-width, so offsets carry over).
      input?.setSelectionRange(caret, caret);
    });
  }

  function key(e: KeyboardEvent) {
    if (e.key === "ArrowUp" || e.key === "ArrowDown") {
      e.preventDefault();
      step(e.key === "ArrowUp" ? 1 : -1, e.shiftKey);
    } else if (e.key === "Enter") {
      e.preventDefault();
      commit();
      input?.select();
    } else if (e.key === "Escape") {
      text = formatTime(value, withHours);
      input?.blur();
    }
  }

  function wheel(e: WheelEvent) {
    if (document.activeElement !== input) return;
    e.preventDefault();
    step(e.deltaY < 0 ? 1 : -1, e.shiftKey);
  }

  $effect(() => {
    const el = input;
    if (!el) return;
    el.addEventListener("wheel", wheel, { passive: false });
    return () => el.removeEventListener("wheel", wheel);
  });
</script>

<input
  bind:this={input}
  class="field time" class:invalid type="text" inputmode="decimal" spellcheck="false" autocomplete="off"
  aria-label={label} {disabled}
  bind:value={text}
  onfocus={() => (editing = true)}
  onblur={() => { editing = false; commit(); }}
  onkeydown={key}
/>

<style>
  .time { width: 100%; font-size: 13px; letter-spacing: .01em; }
</style>
