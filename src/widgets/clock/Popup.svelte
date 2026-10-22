<script lang="ts">
  import { ChevronLeft, ChevronRight } from '@lucide/svelte';
  import type { PopupProps } from '../types';
  import type { ClockSettings } from './index';
  import { fmtTime, ticker } from './time.svelte';

  let { settings }: PopupProps<ClockSettings> = $props();
  const clock = ticker(() => true);
  const today = new Date();
  let month = $state(new Date(today.getFullYear(), today.getMonth(), 1));

  // Monday-first month grid, padded with blanks.
  const cells = $derived.by(() => {
    const lead = (month.getDay() + 6) % 7;
    const days = new Date(month.getFullYear(), month.getMonth() + 1, 0).getDate();
    return [...Array(lead).fill(0), ...Array.from({ length: days }, (_, i) => i + 1)];
  });
  const isToday = (d: number) =>
    d === today.getDate() && month.getMonth() === today.getMonth() && month.getFullYear() === today.getFullYear();
  const shift = (n: number) => (month = new Date(month.getFullYear(), month.getMonth() + n, 1));
  const weekdays = Array.from({ length: 7 }, (_, i) =>
    new Intl.DateTimeFormat(undefined, { weekday: 'narrow' }).format(new Date(2024, 0, 1 + i)));
</script>

<div class="head">
  <div class="big">{fmtTime(clock.now, settings.hour12, true)}</div>
  <div class="dim">{new Intl.DateTimeFormat(undefined, { dateStyle: 'full' }).format(clock.now)}</div>
  {#if settings.secondTz}
    <div class="dim">{settings.secondTz}: {fmtTime(clock.now, settings.hour12, false, settings.secondTz)}</div>
  {/if}
</div>

<div class="card cal">
  <div class="nav">
    <button class="icon-btn" onclick={() => shift(-1)} aria-label="Previous month"><ChevronLeft size={16} /></button>
    <button class="title" onclick={() => (month = new Date(today.getFullYear(), today.getMonth(), 1))}>
      {new Intl.DateTimeFormat(undefined, { month: 'long', year: 'numeric' }).format(month)}
    </button>
    <button class="icon-btn" onclick={() => shift(1)} aria-label="Next month"><ChevronRight size={16} /></button>
  </div>
  <div class="grid">
    {#each weekdays as w, i (i)}<span class="wd dim">{w}</span>{/each}
    {#each cells as d, i (i)}
      <span class="day" class:today={d && isToday(d)}>{d || ''}</span>
    {/each}
  </div>
</div>

<style>
  .head { padding: 2px 4px 12px; }
  .big { font-size: 30px; font-weight: 600; font-variant-numeric: tabular-nums; line-height: 1.1; }
  .cal { padding: 10px; }
  .nav { display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px; }
  .title { font-weight: 600; padding: 4px 10px; border-radius: 10px; }
  .title:hover { background: var(--pill-hover); }
  .grid { display: grid; grid-template-columns: repeat(7, 1fr); gap: 2px; text-align: center; }
  .wd { font-size: 11px; padding: 4px 0; }
  .day { padding: 6px 0; border-radius: 50%; font-variant-numeric: tabular-nums; }
  .today { background: var(--accent); color: #fff; font-weight: 600; }
</style>
