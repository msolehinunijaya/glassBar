<script lang="ts">
  // Circular progress ring. `value` is 0..1.
  let { value, size = 22, stroke = 3, color = 'var(--accent)', children }: {
    value: number; size?: number; stroke?: number; color?: string; children?: import('svelte').Snippet;
  } = $props();
  const r = $derived((size - stroke) / 2);
  const c = $derived(2 * Math.PI * r);
</script>

<span class="ring" style:width="{size}px" style:height="{size}px">
  <svg width={size} height={size} viewBox="0 0 {size} {size}" aria-hidden="true">
    <circle cx={size / 2} cy={size / 2} {r} fill="none" stroke="var(--track)" stroke-width={stroke} />
    <circle cx={size / 2} cy={size / 2} {r} fill="none" stroke={color} stroke-width={stroke} stroke-linecap="round"
            stroke-dasharray={c} stroke-dashoffset={c * (1 - Math.min(1, Math.max(0, value)))}
            transform="rotate(-90 {size / 2} {size / 2})" style="transition: stroke-dashoffset .9s linear" />
  </svg>
  {#if children}<span class="mid">{@render children()}</span>{/if}
</span>

<style>
  .ring { position: relative; display: inline-grid; place-items: center; flex: none; }
  .mid { position: absolute; inset: 0; display: grid; place-items: center; text-align: center; }
</style>
