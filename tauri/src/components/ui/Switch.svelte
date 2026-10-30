<script lang="ts">
  let { checked, onchange, label, disabled = false }: {
    checked: boolean; onchange: (v: boolean) => void; label: string; disabled?: boolean;
  } = $props();
</script>

<button type="button" role="switch" aria-checked={checked} aria-label={label} {disabled}
  class="switch" onclick={() => onchange(!checked)}><span></span></button>

<style>
  .switch {
    flex: none; width: 32px; height: 18px; padding: 0; border-radius: 999px; position: relative;
    border: 1px solid var(--border-strong); background: var(--surface-2);
    transition: background var(--fast), border-color var(--fast);
  }
  span {
    position: absolute; top: 2px; left: 2px; width: 12px; height: 12px; border-radius: 50%;
    background: var(--text-2); transition: transform var(--med) var(--ease), background var(--fast);
  }
  .switch[aria-checked="true"] { background: var(--accent); border-color: var(--accent); }
  .switch[aria-checked="true"] span { transform: translateX(14px); background: var(--accent-ink); }
  .switch:disabled { opacity: .45; }
</style>
