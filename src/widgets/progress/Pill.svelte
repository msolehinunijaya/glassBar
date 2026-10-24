<script lang="ts">
  // A miniature of the reference card: the dot grid and the percentage.
  // Updates once a minute (day) or once an hour (month/year).
  import type { PillProps } from '../types';
  import type { ProgressSettings } from './index';
  import DotGrid from './DotGrid.svelte';
  import { label, period } from './period';

  let { size, settings, vertical }: PillProps<ProgressSettings> = $props();
  let now = $state(new Date());
  $effect(() => {
    const every = settings.mode === 'day' ? 60_000 : 3_600_000;
    const t = setInterval(() => (now = new Date()), every);
    now = new Date();
    return () => clearInterval(t);
  });
  const p = $derived(period(now, settings.mode));
  const year = $derived(settings.mode === 'year');
  // Big enough to read on the dock (3 rows day/month, 4 rows year, still within the bar);
  // smaller on a left/right dock.
  const dot = $derived(vertical ? (year ? 2 : 3) : year ? 5 : 7);
  const KIND = { day: 'Day', month: 'Month', year: 'Year' } as const;
</script>

<div class="pill prog" title="{label(now, settings.mode)}: {Math.floor(p.frac * 100)}% passed">
  <!-- Which period (Day / Month / Year) and how far through it, on the left; dots on the right. -->
  <span class="side">
    <span class="kind">{KIND[settings.mode]}</span>
    <b>{Math.floor(p.frac * 100)}%</b>
  </span>
  <DotGrid mode={settings.mode} frac={p.frac} {dot} gap={vertical ? 1 : year ? 2 : 3} />
</div>

<style>
  .prog { gap: 9px; }
  .side { display: flex; flex-direction: column; line-height: 1.15; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .kind { font-size: 10.5px; color: var(--fg-dim); }
  .side b { font-size: 12.5px; }
</style>
