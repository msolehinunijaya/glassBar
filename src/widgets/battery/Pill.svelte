<script lang="ts">
  // Compact: the percentage and the charging state live inside the battery icon itself.
  // Medium size adds a short word ("Charging", "Full", "Plugged in", "On battery").
  import { ipc, type Battery } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import type { PillProps } from '../types';
  import BatteryIcon from './BatteryIcon.svelte';
  import type { BatterySettings } from './index';
  import LiquidBattery from './LiquidBattery.svelte';

  let { size, settings }: PillProps<BatterySettings> = $props();
  const bat = live<Battery | null>('battery', ipc.batteryGet, null);
  const word = $derived(!bat.v ? 'No battery' : bat.v.charging ? 'Charging' : bat.v.onAc && bat.v.percent >= 95 ? 'Full'
    : bat.v.onAc ? 'Plugged in' : 'On battery');
  import { Plug } from '@lucide/svelte';
</script>

<div class="pill bat-pill" title={bat.v ? `${bat.v.percent}% · ${word}` : 'No battery found (AC power)'}>
  {#if bat.v && settings.style === 'liquid'}
    <LiquidBattery b={bat.v} />
  {:else}
    <BatteryIcon b={bat.v} size={size === 'small' ? 16 : 18} />
  {/if}
  {#if bat.v?.onAc}<span class="plug"><Plug size={13} strokeWidth={2.2} /></span>{/if}
  {#if size !== 'small'}<span class="dim txt">{word}</span>{/if}
</div>

<style>
  .bat-pill { gap: 6px; }
  .txt { font-size: 11px; }
  .plug { display: inline-flex; color: #30d158; margin-left: -2px; }
</style>
