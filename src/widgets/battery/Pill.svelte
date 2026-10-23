<script lang="ts">
  import { ipc, type Battery } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import type { PillProps } from '../types';
  import BatteryIcon from './BatteryIcon.svelte';

  let { size }: PillProps = $props();
  const bat = live<Battery | null>('battery', ipc.batteryGet, null);
</script>

<div class="pill" title={bat.v ? `${bat.v.percent}%${bat.v.charging ? ', charging' : ''}` : 'AC power'}>
  <BatteryIcon b={bat.v} />
  {#if bat.v}<span class="n">{bat.v.percent}%</span>{:else if size !== 'small'}<span class="dim">AC power</span>{/if}
</div>

<style>
  .n { font-variant-numeric: tabular-nums; font-weight: 500; }
</style>
