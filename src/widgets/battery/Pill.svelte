<script lang="ts">
  import { Plug, Unplug, Zap } from '@lucide/svelte';
  import { ipc, type Battery } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import type { PillProps } from '../types';
  import BatteryIcon from './BatteryIcon.svelte';
  import type { BatterySettings } from './index';
  import LiquidBattery from './LiquidBattery.svelte';

  let { size, settings }: PillProps<BatterySettings> = $props();
  const bat = live<Battery | null>('battery', ipc.batteryGet, null);
  // Power source at a glance: charging / plugged in (full or held) / unplugged.
  const source = $derived(!bat.v ? null : bat.v.charging ? 'charging' : bat.v.onAc ? 'plugged' : 'battery');
  const label = { charging: 'Charging', plugged: 'Plugged in, not charging', battery: 'Unplugged, on battery' } as const;
</script>

<div class="pill" title={bat.v && source ? `${bat.v.percent}% · ${label[source]}` : 'AC power'}>
  {#if bat.v && settings.style === 'liquid'}
    <LiquidBattery b={bat.v} />
  {:else}
    <BatteryIcon b={bat.v} />
    {#if bat.v}<span class="n">{bat.v.percent}%</span>{:else if size !== 'small'}<span class="dim">AC power</span>{/if}
  {/if}
  {#if source === 'charging'}
    <span class="src charging" aria-label="Charging"><Zap size={13} fill="currentColor" /></span>
  {:else if source === 'plugged'}
    <span class="src" aria-label="Plugged in"><Plug size={13} /></span>
  {:else if source === 'battery'}
    <span class="src dim" aria-label="Unplugged"><Unplug size={13} /></span>
  {/if}
  {#if size !== 'small' && source}<span class="dim txt">{source === 'charging' ? 'Charging' : source === 'plugged' ? 'Plugged in' : 'Unplugged'}</span>{/if}
</div>

<style>
  .n { font-variant-numeric: tabular-nums; font-weight: 500; }
  .src { display: grid; place-items: center; }
  .charging { color: #30d158; }
  .txt { font-size: 11px; }
</style>
