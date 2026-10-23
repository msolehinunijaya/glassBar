<script lang="ts">
  import { Wind } from '@lucide/svelte';
  import type { PillProps } from '../types';
  import { BANDS } from './bands';
  import type { IpuSettings } from './index';
  import { label, liveIpu } from './ipu.svelte';

  let { size, settings }: PillProps<IpuSettings> = $props();
  const ipu = liveIpu(() => settings.station);
</script>

<div class="pill" title={ipu.r ? `${label(ipu.r)} ${ipu.r.value} · ${BANDS[ipu.r.band].name} · ${ipu.r.station?.name ?? ''}` : 'Air quality'}>
  {#if ipu.r}
    <span class="dot" style:background={BANDS[ipu.r.band].color}></span>
    <span class="v">{Math.round(ipu.r.value)}</span>
    <span class="dim lbl">{label(ipu.r)}</span>
    {#if size !== 'small'}<span class="dim">{BANDS[ipu.r.band].name}</span>{/if}
  {:else}
    <Wind size={16} class="dim" />
  {/if}
</div>

<style>
  .dot { width: 10px; height: 10px; border-radius: 50%; box-shadow: 0 0 0 2px rgba(255,255,255,.25); }
  .v { font-weight: 600; font-variant-numeric: tabular-nums; }
  .lbl { font-size: 11px; }
</style>
