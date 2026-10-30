<script lang="ts" generics="T extends string">
  import Icon from "./Icon.svelte";

  let {
    value, options, onchange, label, size = "md", disabled = false,
  }: {
    value: T;
    options: { value: T; label?: string; icon?: string; tip?: string }[];
    onchange: (v: T) => void;
    label: string;
    size?: "sm" | "md";
    disabled?: boolean;
  } = $props();

  function key(e: KeyboardEvent) {
    const i = options.findIndex((o) => o.value === value);
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
    if (!d) return;
    e.preventDefault();
    const next = options[(i + d + options.length) % options.length];
    onchange(next.value);
    ((e.currentTarget as HTMLElement).querySelector(`[data-v="${next.value}"]`) as HTMLElement | null)?.focus();
  }
</script>

<div class="seg {size}" role="radiogroup" aria-label={label} tabindex="-1" onkeydown={key}>
  {#each options as o (o.value)}
    <button
      type="button" role="radio" aria-checked={o.value === value} data-v={o.value}
      tabindex={o.value === value ? 0 : -1} data-tip={o.tip} {disabled}
      aria-label={o.label ? undefined : o.tip}
      onclick={() => onchange(o.value)}
    >
      {#if o.icon}<Icon name={o.icon} size={size === "sm" ? 14 : 15} />{/if}
      {#if o.label}<span>{o.label}</span>{/if}
    </button>
  {/each}
</div>

<style>
  .seg {
    display: flex; padding: 2px; gap: 2px; border-radius: var(--r-2);
    background: var(--surface-2); border: 1px solid var(--border);
  }
  button {
    flex: 1; display: inline-flex; align-items: center; justify-content: center; gap: 6px;
    height: 24px; padding: 0 10px; border: 0; border-radius: var(--r-1);
    background: transparent; color: var(--text-2); font-weight: 500; white-space: nowrap;
    transition: background var(--fast), color var(--fast), box-shadow var(--fast);
  }
  .sm button { height: 22px; padding: 0 7px; font-size: 12px; }
  button:hover:not(:disabled) { color: var(--text); }
  button[aria-checked="true"] { background: var(--surface); color: var(--text); box-shadow: var(--shadow-1), 0 0 0 1px var(--border); }
  button:disabled { opacity: .5; }
</style>
