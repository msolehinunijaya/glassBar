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
  // Fits the bar: smaller dots on a left/right dock.
  const dot = $derived(vertical ? (year ? 2 : 3) : year ? 3 : 4);
</script>

<div class="pill prog" title="{label(now, settings.mode)}: {Math.floor(p.frac * 100)}% passed">
  <DotGrid mode={settings.mode} frac={p.frac} {dot} gap={vertical ? 1 : year ? 1.5 : 2} />
  <span class="txt">{#if size !== 'small' && !vertical}<span class="dim">{label(now, settings.mode)}</span> {/if}<b>{Math.floor(p.frac * 100)}%</b></span>
</div>

<style>
  .prog { gap: 8px; }
  .txt { font-size: 12px; font-variant-numeric: tabular-nums; white-space: nowrap; }
</style>
