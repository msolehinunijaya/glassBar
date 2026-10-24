<script lang="ts">
  import { Hourglass } from '@lucide/svelte';
  import { ipc, type ScreenToday } from '../../lib/ipc';
  import { duration } from '../../lib/time';
  import type { PillProps } from '../types';
  import type { ScreenTimeSettings } from './index';

  let { size, settings, vertical }: PillProps<ScreenTimeSettings> = $props();
  // Last 7 days for the mini graph (today is the last bar).
  let week = $state<[string, number][]>([]);
  const weekMax = $derived(Math.max(1, ...week.map((d) => d[1])));
  let today = $state<ScreenToday | null>(null);
  // Rust saves once a minute, so a minute refresh is as fresh as it gets.
  $effect(() => {
    const load = () => {
      ipc.screentimeToday().then((t) => (today = t));
      if (settings.showGraph) ipc.screentimeWeek().then((w) => (week = w.slice(-7))).catch(() => {});
    };
    load();
    const t = setInterval(load, 60_000);
    return () => clearInterval(t);
  });
</script>

<div class="pill" title="Screen time today">
  <span class="stack">
    <Hourglass size={14} />
    <span class="t">{duration(today?.total ?? 0)}</span>
  </span>
  {#if settings.showGraph && !vertical && week.length}
    <span class="mini" title="Last 7 days">
      {#each week as [d, s], i (d)}<i class:today={i === week.length - 1} style:height="{Math.max(8, (s / weekMax) * 100)}%"></i>{/each}
    </span>
  {/if}
  {#if size === 'large' && today?.apps[0]}<span class="dim">{today.apps[0].name}</span>{/if}
</div>

<style>
  .stack { display: flex; flex-direction: column; align-items: center; line-height: 1.05; gap: 2px; }
  .t { font-weight: 600; font-size: 11.5px; font-variant-numeric: tabular-nums; white-space: nowrap; }
  /* Mini 7-day graph: today in the accent colour. */
  .mini { display: flex; align-items: flex-end; gap: 2px; height: 22px; }
  .mini i { width: 4px; border-radius: calc(2px * var(--rs)); background: color-mix(in srgb, var(--fg) 30%, transparent); transition: height 0.4s var(--ease); }
  .mini i.today { background: var(--accent); }
</style>
