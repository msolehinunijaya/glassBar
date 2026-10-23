<script lang="ts">
  import { AlarmClock, Watch } from '@lucide/svelte';
  import { ipc, type Stopwatch, type Timer } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { tickingNow } from '../../lib/now.svelte';
  import { clock } from '../../lib/time';
  import type { PillProps } from '../types';

  let { size }: PillProps = $props();
  const timers = live<Timer[]>('timers', ipc.timersList, []);
  const sw = live<Stopwatch | null>('stopwatch', ipc.swGet, null);
  const t = tickingNow(() => timers.v.some((x) => x.endsAt && x.endsAt > Date.now()) || !!sw.v?.startedAt);
  const next = $derived(timers.v.filter((x) => x.endsAt && x.endsAt > t.now).sort((a, b) => a.endsAt! - b.endsAt!)[0]);
</script>

<div class="pill">
  {#if next}
    <AlarmClock size={16} /><span class="t">{clock(next.endsAt! - t.now)}</span>
    {#if size !== 'small' && next.label}<span class="dim ellipsis lbl">{next.label}</span>{/if}
  {:else if sw.v?.startedAt}
    <Watch size={16} /><span class="t">{clock(sw.v.accMs + t.now - sw.v.startedAt)}</span>
  {:else}
    <AlarmClock size={16} />
  {/if}
</div>

<style>
  .t { font-variant-numeric: tabular-nums; font-weight: 600; }
  .lbl { max-width: 90px; font-size: 12px; }
</style>
