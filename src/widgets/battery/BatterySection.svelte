<script lang="ts">
  import BatteryFill from './BatteryFill.svelte';
  import { Gauge, Leaf, Rocket, Settings2 } from '@lucide/svelte';
  import { ipc, type Battery, type PowerInfo, type PowerMode } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { duration } from '../../lib/time';
  import BatteryIcon from './BatteryIcon.svelte';
  import LiquidBattery from './LiquidBattery.svelte';
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';

  // Battery panel (from the battery segment of Quick Settings): charge, design, power mode,
  // battery saver. The battery always shows on the dock when the PC has one.
  let { instance, style }: { instance: string; style: 'classic' | 'liquid' | 'fill' } = $props();
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
    try { await ipc.powerSetMode(m); } catch (e) { error = String(e); }
    power = await ipc.powerGet();
  }
  async function useBalanced() {
    error = '';
    try { await ipc.powerUseBalanced(); } catch (e) { error = String(e); }
    power = await ipc.powerGet();
  }
</script>

<div class="bhead"><h4 class="dim">Battery</h4></div>
{#if bat.v}
  <div class="big">
    {#if style === 'liquid'}<LiquidBattery b={bat.v} height={40} />{:else if style === 'fill'}<span class="fillbox"><BatteryFill b={bat.v} big /></span>{:else}<BatteryIcon b={bat.v} size={30} />{/if}
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
    { type: 'choice', key: 'batteryStyle', label: 'Design', choices: [{ value: 'classic', label: 'Classic' }, { value: 'liquid', label: 'Liquid' }, { value: 'fill', label: 'Fill', hint: 'The whole battery part fills up, % in the middle' }] },
  ]} />
</div>

<h4 class="dim">Power mode</h4>
<div class="seg" class:off={!!power?.otherPlan}>
  {#each modes as [m, label, Icon] (m)}
    <button class:sel={!power?.otherPlan && power?.mode === m} disabled={!!power?.otherPlan} onclick={() => setMode(m)}><Icon size={15} /><span>{label}</span></button>
  {/each}
</div>
{#if power?.otherPlan}
  <div class="plan card">
    <span class="small">Power mode only works with Windows' <b>Balanced</b> power plan. This PC uses {power.otherPlan}.</span>
    <button class="btn" onclick={useBalanced}>Use Balanced plan</button>
  </div>
{/if}

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
  .big { display: flex; align-items: center; gap: 12px; font-size: 26px; font-weight: 600; }
  .status { margin: 4px 0 0; }
  .style { margin-top: 10px; }
  .fillbox { position: relative; display: grid; place-items: center; width: 120px; height: 44px; border-radius: var(--r-inner); overflow: hidden;
             background: var(--bg-2, var(--pill)); border: 1px solid var(--line-soft); }
  .seg { display: grid; grid-template-columns: repeat(3, 1fr); gap: 3px; padding: 3px; border-radius: calc(12px * var(--rs)); background: var(--pill); }
  .seg button { display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 8px 2px; border-radius: calc(9px * var(--rs)); font-size: 11px; }
  .seg .sel { background: var(--accent); color: var(--on-accent); }
  .seg button:not(:disabled):hover:not(.sel) { background: var(--pill-hover); }
  .seg.off { opacity: 0.45; }
  .plan { display: flex; flex-direction: column; gap: 8px; margin-top: 8px; padding: 10px 12px; }
  .plan .btn { align-self: flex-start; padding: 6px 12px; border-radius: calc(10px * var(--rs)); background: var(--accent); color: var(--on-accent); font-size: 12px; }
  .saver { display: flex; align-items: center; gap: 10px; margin-top: 12px; padding: 10px 12px; }
  .col { flex: 1; display: flex; flex-direction: column; }
  .small { font-size: 11px; }
  .note { margin: 6px 2px 0; }
  .err { color: #ff453a; font-size: 12px; }
</style>
