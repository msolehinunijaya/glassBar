<script lang="ts">
  // e.g. "Day 72%" with a slim bar underneath. Updates once a minute (day mode) or
  // once an hour (month/year): no animation, nothing busy.
  import type { PillProps } from '../types';
  import type { ProgressSettings } from './index';
  import { period } from './period';

  let { size, settings }: PillProps<ProgressSettings> = $props();
  let now = $state(new Date());
  $effect(() => {
    const every = settings.mode === 'day' ? 60_000 : 3_600_000;
    const t = setInterval(() => (now = new Date()), every);
    now = new Date();
    return () => clearInterval(t);
  });
  const p = $derived(period(now, settings.mode));
  const name = $derived(settings.mode === 'day' ? 'Day' : settings.mode === 'month' ? 'Month' : 'Year');
</script>

<div class="pill prog" title="{name}: {Math.floor(p.frac * 100)}% passed">
  <div class="col">
    <span class="txt">{#if size !== 'small'}<span class="dim">{name}</span> {/if}<b>{Math.floor(p.frac * 100)}%</b></span>
    <span class="track"><span class="fill" style:width="{p.frac * 100}%"></span></span>
  </div>
</div>

<style>
  .col { display: flex; flex-direction: column; gap: 3px; min-width: 52px; }
  .txt { font-size: 12px; font-variant-numeric: tabular-nums; }
  .track { height: 4px; border-radius: 2px; background: var(--track); overflow: hidden; }
  .fill { display: block; height: 100%; border-radius: 2px; background: var(--accent); transition: width 0.6s var(--ease); }
</style>
