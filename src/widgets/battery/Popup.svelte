<script lang="ts">
  import { ipc, type Battery } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { duration } from '../../lib/time';
  import type { PopupProps } from '../types';
  import BatteryIcon from './BatteryIcon.svelte';

  let {}: PopupProps = $props();
  const bat = live<Battery | null>('battery', ipc.batteryGet, null);
</script>

<h3>Battery</h3>
{#if bat.v}
  <div class="big"><BatteryIcon b={bat.v} size={34} /> {bat.v.percent}%</div>
  <div class="bar"><div style:width="{bat.v.percent}%" class:low={bat.v.percent <= 15 && !bat.v.charging}></div></div>
  <p class="dim">
    {#if bat.v.charging}Charging{:else if bat.v.onAc}Plugged in{:else}On battery{/if}
    {#if bat.v.secsLeft && !bat.v.onAc} · about {duration(bat.v.secsLeft)} left{/if}
  </p>
{:else}
  <p class="dim">This PC runs on AC power (no battery found).</p>
{/if}

<style>
  h3 { margin: 0 0 10px; font-size: 15px; font-weight: 600; }
  .big { display: flex; align-items: center; gap: 10px; font-size: 34px; font-weight: 600; }
  .bar { height: 10px; border-radius: 5px; background: var(--track); overflow: hidden; margin: 12px 0 8px; }
  .bar div { height: 100%; background: #30d158; border-radius: 5px; }
  .bar .low { background: #ff453a; }
</style>
