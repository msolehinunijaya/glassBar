<script lang="ts">
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import type { PopupProps } from '../types';
  import type { ProgressSettings } from './index';
  import { period, span, type Mode } from './period';

  let { instance, settings }: PopupProps<ProgressSettings> = $props();
  let now = $state(new Date());
  $effect(() => { const t = setInterval(() => (now = new Date()), 30_000); return () => clearInterval(t); });
  const p = $derived(period(now, settings.mode));
  const fmt = (ms: number, opts: Intl.DateTimeFormatOptions) => new Intl.DateTimeFormat(undefined, opts).format(ms);
  const title = $derived(settings.mode === 'day' ? fmt(p.now, { weekday: 'long', day: 'numeric', month: 'long' })
    : settings.mode === 'month' ? fmt(p.now, { month: 'long', year: 'numeric' }) : String(new Date(p.now).getFullYear()));
  const modes: [Mode, string][] = [['day', 'Day'], ['month', 'Month'], ['year', 'Year']];
</script>

<div class="seg">
  {#each modes as [m, label] (m)}
    <button class:sel={settings.mode === m} onclick={() => updateWidgetSettings(instance, { mode: m })}>{label}</button>
  {/each}
</div>
<h3>{title}</h3>
<div class="big">{(p.frac * 100).toFixed(1)}<span>%</span></div>
<div class="track"><span class="fill" style:width="{p.frac * 100}%"></span></div>
{#if settings.showTimes}
  <div class="times">
    <div><span class="dim">Elapsed</span><b>{span(p.now - p.start)}</b></div>
    <div><span class="dim">Remaining</span><b>{span(p.end - p.now)}</b></div>
  </div>
{/if}
{#if settings.mode === 'day'}<p class="dim small">Now {fmt(p.now, { hour: '2-digit', minute: '2-digit' })} · resets at midnight</p>{/if}

<style>
  .seg { display: grid; grid-template-columns: repeat(3, 1fr); gap: 2px; padding: 3px; border-radius: 10px; background: var(--pill); margin-bottom: 12px; }
  .seg button { padding: 6px; border-radius: 8px; font-size: 12px; }
  .seg .sel { background: var(--accent); color: #fff; }
  h3 { margin: 0; font-size: 14px; font-weight: 500; }
  .big { font-size: 40px; font-weight: 700; font-variant-numeric: tabular-nums; line-height: 1.1; }
  .big span { font-size: 20px; opacity: 0.6; margin-left: 2px; }
  .track { height: 10px; border-radius: 5px; background: var(--track); overflow: hidden; margin: 8px 0 12px; }
  .fill { display: block; height: 100%; border-radius: 5px; background: var(--accent); }
  .times { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .times div { display: flex; flex-direction: column; padding: 8px 10px; border-radius: 12px; background: var(--pill); font-size: 12px; }
  .times b { font-size: 15px; }
  .small { font-size: 11px; margin: 10px 0 0; }
</style>
