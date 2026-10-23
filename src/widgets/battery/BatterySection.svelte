<script lang="ts">
  import { Gauge, Leaf, Rocket, Settings2 } from '@lucide/svelte';
  import { ipc, type Battery, type PowerInfo, type PowerMode } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { duration } from '../../lib/time';
  import BatteryIcon from './BatteryIcon.svelte';
  import LiquidBattery from './LiquidBattery.svelte';
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';

  // Battery section of the Quick Settings panel: charge, design, power mode, battery saver,
  // and the switch that shows the battery on the dock (after the Bluetooth icon).
  let { instance, style, onDock }: { instance: string; style: 'classic' | 'liquid'; onDock: boolean } = $props();
  const bat = live<Battery | null>('battery', ipc.batteryGet, null);
  let power = $state<PowerInfo | null>(null);
  let error = $state('');
  $effect(() => { ipc.powerGet().then((p) => (power = p)); });

  // The same three modes as Settings → System → Power & battery → Power mode.
  const modes: [PowerMode, string, typeof Leaf][] = [
    ['efficiency', 'Best efficiency', Leaf],
    ['balanced', 'Balanced', Gauge],
    ['performance', 'Best performance', Rocket],
  ];
  async function setMode(m: PowerMode) {
    error = '';
    try { await ipc.powerSetMode(m); power = await ipc.powerGet(); } catch (e) { error = String(e); }
  }
</script>

<div class="bhead">
  <h4 class="dim">Battery</h4>
  <label class="dim sw">Show on dock
    <input type="checkbox" class="switch" checked={onDock} onchange={(e) => updateWidgetSettings(instance, { batteryOnDock: e.currentTarget.checked })} />
  </label>
</div>
{#if bat.v}
  <div class="big">
    {#if style === 'liquid'}<LiquidBattery b={bat.v} height={40} />{:else}<BatteryIcon b={bat.v} size={30} />{/if}
    <span>{bat.v.percent}%</span>
  </div>
  <p class="dim status">
    {#if bat.v.charging}Charging{:else if bat.v.onAc}Plugged in{:else}On battery{/if}
    {#if bat.v.secsLeft && !bat.v.onAc} · about {duration(bat.v.secsLeft)} left{/if}
  </p>
{:else}
  <p class="dim">This PC runs on AC power (no battery found).</p>
{/if}

<div class="style">
  <OptionsPopover title="Battery icon" values={{ batteryStyle: style }} save={(p) => updateWidgetSettings(instance, p)} options={[
    { type: 'choice', key: 'batteryStyle', label: 'Design', choices: [{ value: 'classic', label: 'Classic' }, { value: 'liquid', label: 'Liquid' }] },
  ]} />
</div>

<h4 class="dim">Power mode</h4>
<div class="seg">
  {#each modes as [m, label, Icon] (m)}
    <button class:sel={power?.mode === m} onclick={() => setMode(m)}><Icon size={15} /><span>{label}</span></button>
  {/each}
</div>

<div class="saver card">
  <Leaf size={16} />
  <div class="col">
    <b>Battery saver</b>
    <span class="dim small">{power?.saverOn ? 'On: background activity is limited' : power?.saverOn === false ? 'Off' : 'Status unavailable'}</span>
  </div>
  <button class="btn" onclick={() => ipc.appLaunch('ms-settings:batterysaver')}><Settings2 size={14} /> Settings</button>
</div>
<p class="dim small note">Windows only lets its own Settings switch battery saver on or off.</p>
{#if error}<p class="err">{error}</p>{/if}

<style>
  h4 { margin: 14px 2px 6px; font-size: 12px; font-weight: 500; }
  .bhead { display: flex; align-items: center; justify-content: space-between; margin: 14px 2px 6px; }
  .bhead h4 { margin: 0; }
  .sw { display: flex; align-items: center; gap: 6px; font-size: 11px; }
  .big { display: flex; align-items: center; gap: 12px; font-size: 26px; font-weight: 600; }
  .status { margin: 4px 0 0; }
  .style { margin-top: 10px; }
  .seg { display: grid; grid-template-columns: repeat(3, 1fr); gap: 3px; padding: 3px; border-radius: 12px; background: var(--pill); }
  .seg button { display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 8px 2px; border-radius: 9px; font-size: 11px; }
  .seg .sel { background: var(--accent); color: #fff; }
  .saver { display: flex; align-items: center; gap: 10px; margin-top: 12px; padding: 10px 12px; }
  .col { flex: 1; display: flex; flex-direction: column; }
  .small { font-size: 11px; }
  .note { margin: 6px 2px 0; }
  .err { color: #ff453a; font-size: 12px; }
</style>
