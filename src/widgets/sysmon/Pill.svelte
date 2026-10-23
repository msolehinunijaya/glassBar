<script lang="ts">
  // Shows only the items ticked in the widget's options (CPU / RAM / Network).
  // Items slide in and out, so the pill's width changes smoothly.
  import { slide } from 'svelte/transition';
  import Sparkline from '../../lib/Sparkline.svelte';
  import type { PillProps } from '../types';
  import type { SysmonSettings } from './index';
  import { liveSamples, rate } from './samples.svelte';
  import { fmtTemp, isHot } from './temp';

  let { size, settings }: PillProps<SysmonSettings> = $props();
  const s = liveSamples();
  const last = $derived(s.list.at(-1));
  const graphs = $derived(size !== 'small');
  const t = { axis: 'x' as const, duration: 250 };
</script>

<div class="pill" title="System">
  {#if settings.cpu}
    <span class="m" transition:slide={t}><span class="dim k">CPU</span>{Math.round(last?.cpu ?? 0)}%{#if settings.temp}<span class="dim">·</span><span class:hot={isHot(last?.cpuTemp, +settings.tempWarn)}>{fmtTemp(last?.cpuTemp, settings.tempUnit)}</span>{/if}
      {#if graphs}<Sparkline values={s.list.map((x) => x.cpu)} width={36} height={16} />{/if}</span>
  {/if}
  {#if settings.ram}
    <span class="m" transition:slide={t}><span class="dim k">RAM</span>{Math.round(last?.mem ?? 0)}%
      {#if graphs}<Sparkline values={s.list.map((x) => x.mem)} width={36} height={16} color="#bf5af2" />{/if}</span>
  {/if}
  {#if settings.network && last}
    <span class="m net" transition:slide={t}><span class="dim k">↓</span>{rate(last.rxBps)} <span class="dim k">↑</span>{rate(last.txBps)}</span>
  {/if}
</div>

<style>
  .m { display: inline-flex; align-items: center; gap: 4px; font-variant-numeric: tabular-nums; font-size: 12px; font-weight: 500; white-space: nowrap; }
  .k { font-size: 10px; }
  .net { min-width: 120px; }
  .hot { color: #ff453a; font-weight: 600; }
</style>
