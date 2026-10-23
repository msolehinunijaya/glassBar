<script lang="ts">
  import { Hourglass } from '@lucide/svelte';
  import { ipc, type ScreenToday } from '../../lib/ipc';
  import { duration } from '../../lib/time';
  import type { PillProps } from '../types';

  let { size }: PillProps = $props();
  let today = $state<ScreenToday | null>(null);
  // Rust saves once a minute, so a minute refresh is as fresh as it gets.
  $effect(() => {
    const load = () => ipc.screentimeToday().then((t) => (today = t));
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
  {#if size === 'large' && today?.apps[0]}<span class="dim">{today.apps[0].name}</span>{/if}
</div>

<style>
  .stack { display: flex; flex-direction: column; align-items: center; line-height: 1.05; gap: 2px; }
  .t { font-weight: 600; font-size: 11.5px; font-variant-numeric: tabular-nums; white-space: nowrap; }
</style>
