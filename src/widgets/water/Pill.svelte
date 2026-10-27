<script lang="ts">
  // A drop that fills with today's water, and the amount (or %) beside it. Click for the panel.
  import type { PillProps } from '../types';
  import type { WaterSettings } from './settings';
  import { fmtWater, progressOf } from './units';
  import { liveWater } from './live.svelte';
  import Drop from './Drop.svelte';

  let { size, settings, vertical }: PillProps<WaterSettings> = $props();
  const w = liveWater(() => settings);
  const p = $derived(w.v ? progressOf(w.v.total, w.v.goal) : 0);
  const text = $derived(!w.v ? '' : settings.show === 'percent' ? `${Math.round(p * 100)}%` : fmtWater(w.v.total, settings.unit));
  const title = $derived(w.v ? `Water today: ${fmtWater(w.v.total, settings.unit)} of ${fmtWater(w.v.goal, settings.unit)}` : 'Water');
</script>

{#if vertical}
  <div class="side" {title}>
    <Drop fill={p} size={20} />
    <span class="side-v">{Math.round(p * 100)}<small>%</small></span>
  </div>
{:else}
  <div class="pill water" class:done={p >= 1} {title}>
    <Drop fill={p} size={18} />
    {#if size !== 'small' && text}<span class="v">{text}</span>{/if}
  </div>
{/if}

<style>
  .water { gap: 6px; padding: 0 10px 0 8px; }
  .v { font-size: 12px; font-weight: 600; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .done .v { color: #40c8e0; }
</style>
