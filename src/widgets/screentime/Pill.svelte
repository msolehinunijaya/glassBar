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
  <Hourglass size={15} />
  <span class="t">{duration(today?.total ?? 0)}</span>
  {#if size !== 'small' && today?.apps[0]}<span class="dim">{today.apps[0].name}</span>{/if}
</div>

<style>
  .t { font-weight: 600; font-variant-numeric: tabular-nums; }
</style>
