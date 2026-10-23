<script lang="ts">
  // Shows only the items ticked in the widget's options (CPU / RAM / Network).
  // Items slide in and out, so the pill's width changes smoothly.
  import { slide } from 'svelte/transition';
  import Sparkline from '../../lib/Sparkline.svelte';
  import type { PillProps } from '../types';
  import type { SysmonSettings } from './index';
  import { liveSamples, rate } from './samples.svelte';
  import { fmtTemp, isHot } from './temp';
  import { letter, level, levelColor, pickDrives, usedPct } from './disk';
  import { liveDrives } from './drives.svelte';

  let { size, settings }: PillProps<SysmonSettings> = $props();
  const s = liveSamples();
  const last = $derived(s.list.at(-1));
  const graphs = $derived(size !== 'small');
  const t = { axis: 'x' as const, duration: 250 };
  // Stacked: the reading on top, its graph underneath.
  const GW = 46, GH = 12;
  // Disk space changes slowly: once a minute is plenty, and only while shown.
  const drives = liveDrives(60_000, () => settings.disk);
  const shownDrives = $derived(settings.disk ? pickDrives(drives.list, settings.drives) : []);
</script>

<div class="pill" title="System">
  {#if settings.cpu}
    <span class="m" class:stack={graphs} transition:slide={t}>
      <span class="row"><span class="dim k">CPU</span>{Math.round(last?.cpu ?? 0)}%{#if settings.temp}<span class="dim">·</span><span class:hot={isHot(last?.cpuTemp, +settings.tempWarn)}>{fmtTemp(last?.cpuTemp, settings.tempUnit)}</span>{/if}</span>
      {#if graphs}<Sparkline values={s.list.map((x) => x.cpu)} width={GW} height={GH} />{/if}</span>
  {/if}
  {#if settings.ram}
    <span class="m" class:stack={graphs} transition:slide={t}>
      <span class="row"><span class="dim k">RAM</span>{Math.round(last?.mem ?? 0)}%</span>
      {#if graphs}<Sparkline values={s.list.map((x) => x.mem)} width={GW} height={GH} color="#bf5af2" />{/if}</span>
  {/if}
  {#each shownDrives as d (d.mount)}
    {@const p = usedPct(d)}
    <span class="m disk stack" transition:slide={t} title="{letter(d.mount)} {p}% used">
      <span class="row"><span class="dim k">{letter(d.mount)}</span>{p}%</span>
      <span class="dtrack"><span style:width="{p}%" style:background={levelColor[level(p, +settings.diskHigh, +settings.diskCritical)]}></span></span>
    </span>
  {/each}
  {#if settings.network && last}
    <span class="m net stack" transition:slide={t} title="Download / upload">
      <span class="row"><span class="dim k">↓</span>{rate(last.rxBps)}</span>
      <span class="row"><span class="dim k">↑</span>{rate(last.txBps)}</span>
    </span>
  {/if}
</div>

<style>
  .m { display: inline-flex; align-items: center; gap: 4px; font-variant-numeric: tabular-nums; font-size: 12px; font-weight: 500; white-space: nowrap; }
  .row { display: inline-flex; align-items: center; gap: 4px; }
  .stack { flex-direction: column; align-items: flex-start; gap: 2px; line-height: 1.1; }
  .stack .row { font-size: 11px; }
  .k { font-size: 10px; }
  .net { min-width: 64px; }
  .net .row { gap: 3px; }
  .dtrack { display: inline-block; width: 46px; height: 4px; border-radius: 2px; background: var(--track); overflow: hidden; }
  .dtrack span { display: block; height: 100%; border-radius: 2px; }
  .hot { color: #ff453a; font-weight: 600; }
</style>
