<script lang="ts">
  // The whole pill takes the status colour (blue → red), so air quality reads at a glance.
  import { Wind } from '@lucide/svelte';
  import type { PillProps } from '../types';
  import { BANDS } from './bands';
  import type { IpuSettings } from './index';
  import { label, liveIpu, selectionKey } from './ipu.svelte';

  let { size, settings }: PillProps<IpuSettings> = $props();
  const ipu = liveIpu(() => selectionKey(settings));
  const band = $derived(ipu.r ? BANDS[ipu.r.band] : null);
  // Dark text on the light colours (yellow, green), white on the rest.
  const ink = $derived(ipu.r && ['unhealthy', 'moderate'].includes(ipu.r.band) ? '#1c1c1e' : '#fff');
</script>

<div class="pill ipu" class:coloured={!!band} style:--c={band?.color} style:--ink={ink}
     title={ipu.r ? `${label(ipu.r)} ${ipu.r.value} · ${band?.name} · ${ipu.r.station?.name ?? ''}` : 'Air quality'}>
  {#if ipu.r}
    <Wind size={14} />
    <span class="v">{Math.round(ipu.r.value)}</span>
    <span class="lbl">{label(ipu.r)}</span>
    {#if size !== 'small'}<span class="name">{band?.name}</span>{/if}
  {:else}
    <Wind size={16} class="dim" />
  {/if}
</div>

<style>
  .ipu.coloured {
    background: color-mix(in srgb, var(--c) 82%, transparent);
    border-color: color-mix(in srgb, var(--c) 100%, transparent);
    color: var(--ink);
  }
  .ipu.coloured:hover { background: var(--c); }
  .v { font-weight: 700; font-variant-numeric: tabular-nums; }
  .lbl { font-size: 10px; opacity: 0.8; }
  .name { font-size: 12px; font-weight: 500; }
</style>
