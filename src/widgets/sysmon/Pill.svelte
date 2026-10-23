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
  import { Trash2 } from '@lucide/svelte';
  import { events, ipc, type RecycleInfo } from '../../lib/ipc';
  import { fmtSize } from '../downloads/format';

  import { loadTone, rateShort } from '../../lib/side';

  let { size, settings, vertical }: PillProps<SysmonSettings> = $props();
  const s = liveSamples();
  const last = $derived(s.list.at(-1));
  const graphs = $derived(size !== 'small');
  const t = $derived({ axis: vertical ? 'y' as const : 'x' as const, duration: 250 });
  // Stacked: the reading on top, its graph underneath.
  const GW = 46, GH = 12;
  // Disk space changes slowly: once a minute is plenty, and only while shown.
  const drives = liveDrives(60_000, () => settings.disk);
  const shownDrives = $derived(settings.disk ? pickDrives(drives.list, settings.drives) : []);
  // Recycle Bin size (optional): once a minute, and right after it changes.
  let bin = $state<RecycleInfo | null>(null);
  $effect(() => {
    if (!settings.recycle) { bin = null; return; }
    const load = () => ipc.recycleInfo().then((i) => (bin = i)).catch(() => (bin = null));
    load();
    const t = setInterval(load, 60_000);
    const off = events.changed('recycle', load);
    return () => { clearInterval(t); off.then((f) => f()); };
  });
</script>

{#if vertical}
<!-- Left/right dock: one tile per reading, value over a load meter. -->
<div class="side-stack" title="System">
  {#if settings.cpu}
    {@const v = Math.round(last?.cpu ?? 0)}
    <div class="side" transition:slide={t}>
      <span class="side-k">CPU</span><span class="side-v">{v}<small>%</small></span>
      <span class="side-meter"><i style:width="{v}%" style:--c={loadTone(v)}></i></span>
      {#if settings.temp}<span class="side-c" class:hot={isHot(last?.cpuTemp, +settings.tempWarn)}>{fmtTemp(last?.cpuTemp, settings.tempUnit)}</span>{/if}
    </div>
  {/if}
  {#if settings.ram}
    {@const v = Math.round(last?.mem ?? 0)}
    <div class="side" transition:slide={t}>
      <span class="side-k">RAM</span><span class="side-v">{v}<small>%</small></span>
      <span class="side-meter"><i style:width="{v}%" style:--c={loadTone(v)}></i></span>
    </div>
  {/if}
  {#each shownDrives as d (d.mount)}
    {@const p = usedPct(d)}
    <div class="side" transition:slide={t} title="{letter(d.mount)} {p}% used">
      <span class="side-k">{letter(d.mount)}</span><span class="side-v">{p}<small>%</small></span>
      <span class="side-meter"><i style:width="{p}%" style:--c={levelColor[level(p, +settings.diskHigh, +settings.diskCritical)]}></i></span>
    </div>
  {/each}
  {#if settings.recycle && bin}
    <div class="side" transition:slide={t} title="Recycle Bin: {bin.items} items"><Trash2 size={14} class="dim" /><span class="side-k">{fmtSize(bin.size)}</span></div>
  {/if}
  {#if settings.network && last}
    <div class="side" transition:slide={t} title="Download {rate(last.rxBps)} · upload {rate(last.txBps)}">
      <span class="side-k">Net</span><span class="side-v net">↓{rateShort(last.rxBps)}</span><span class="side-c">↑{rateShort(last.txBps)}</span>
    </div>
  {/if}
</div>
{:else}
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
  {#if settings.recycle && bin}
    <span class="m" transition:slide={t} title="Recycle Bin: {bin.items} items"><Trash2 size={13} class="dim" />{fmtSize(bin.size)}</span>
  {/if}
  {#if settings.network && last}
    <span class="m net stack" transition:slide={t} title="Download / upload">
      <span class="row"><span class="dim k">↓</span>{rate(last.rxBps)}</span>
      <span class="row"><span class="dim k">↑</span>{rate(last.txBps)}</span>
    </span>
  {/if}
</div>
{/if}

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
  .side-v.net { font-size: 0.85em; }
</style>
