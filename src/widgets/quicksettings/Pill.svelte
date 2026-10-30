<script lang="ts">
  import { Bluetooth, BluetoothOff, ChevronDown, ChevronLeft, ChevronRight, ChevronUp, Plane, Wifi, WifiOff } from '@lucide/svelte';
  import { slide } from 'svelte/transition';
  import { ipc, type RadioInfo } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import type { QuickSettings } from './index';
  import TrayIconButton from '../tray/TrayIconButton.svelte';
  import { liveTray } from '../tray/icons.svelte';
  import { Plug } from '@lucide/svelte';
  import { live } from '../../lib/live.svelte';
  import type { Battery } from '../../lib/ipc';
  import BatteryIcon from '../battery/BatteryIcon.svelte';
  import LiquidBattery from '../battery/LiquidBattery.svelte';
  import BatteryTall from '../battery/BatteryTall.svelte';
  import BatteryFill from '../battery/BatteryFill.svelte';
  import SoundPart from '../sound/SoundPart.svelte';
  import { BATTERY_PANEL, soundOf } from '../sound/settings';
  import { reallyLeft } from '../../lib/hoverPanel';
  import DockMarks from './DockMarks.svelte';

  let { instance, size, settings, vertical }: PillProps<QuickSettings> = $props();
  // Tray icons on the dock (switch in the panel), collapsed behind a chevron by default.
  // Click the chevron to show them all; they fold away again on a click anywhere else on the
  // dock, or 600 ms after the pointer leaves them -- but not while a menu is open (a tray
  // icon's own right-click menu, which also takes focus, so focus can't be the signal).
  const tray = liveTray();
  const trayIcons = $derived(settings.trayOnDock ? [...tray.v].sort((a, b) => Number(b.promoted) - Number(a.promoted)) : []);
  let expanded = $state(false);
  let trayEl = $state<HTMLElement>();
  $effect(() => {
    if (!expanded) return;
    const outside = (e: PointerEvent) => { if (!trayEl?.contains(e.target as Node)) expanded = false; };
    addEventListener('pointerdown', outside, true);
    return () => { removeEventListener('pointerdown', outside, true); clearTimeout(foldTimer); };
  });
  let foldTimer: ReturnType<typeof setTimeout> | undefined;
  async function foldWhenFree() {
    if (await ipc.menuOpen().catch(() => false)) { foldTimer = setTimeout(foldWhenFree, 300); return; }
    expanded = false;
  }
  function trayLeave(e: PointerEvent) {
    if (!expanded || !reallyLeft(e, trayEl)) return; // a tooltip appearing is not a leave
    clearTimeout(foldTimer);
    foldTimer = setTimeout(foldWhenFree, 600);
  }
  const trayEnter = () => clearTimeout(foldTimer);
  // The chevron points the way the icons unfold (towards the screen centre on side docks).
  const Open = $derived(vertical ? ChevronDown : ChevronLeft);
  const Close = $derived(vertical ? ChevronUp : ChevronRight);
  const t = $derived({ axis: (vertical ? 'y' : 'x') as 'x' | 'y', duration: 220 });
  // Keyboard language code on the dock (switch in the panel).
  // ponytail: 2 s poll; Windows has no cheap cross-process "layout changed" event.
  let lang = $state<{ code: string; name: string } | null>(null);
  $effect(() => {
    if (!settings.langOnDock) { lang = null; return; }
    const load = () => ipc.langList().then((l) => { const a = l.find((x) => x.active) ?? l[0]; lang = a ? { code: a.code, name: a.name } : null; }).catch(() => {});
    load();
    const t = setInterval(load, 2000);
    return () => clearInterval(t);
  });
  // Battery after the Bluetooth icon (switch in the panel). Nothing on a PC without one.
  const bat = live<Battery | null>('battery', ipc.batteryGet, null);
  const batWord = $derived(!bat.v ? '' : bat.v.charging ? 'Charging' : bat.v.onAc && bat.v.percent >= 95 ? 'Full' : bat.v.onAc ? 'Plugged in' : 'On battery');
  let radios = $state<RadioInfo[]>([]);
  const has = (k: RadioInfo['kind']) => radios.some((r) => r.kind === k);
  const on = (k: RadioInfo['kind']) => radios.some((r) => r.kind === k && r.on);
  const airplane = $derived(radios.length > 0 && radios.every((r) => !r.on));

  // ponytail: 15 s poll; radios rarely change outside this panel.
  $effect(() => {
    const load = () => ipc.radios().then((r) => (radios = r)).catch(() => {});
    load();
    const t = setInterval(load, 15000);
    return () => clearInterval(t);
  });
</script>

{#if vertical}
<!-- Left/right dock: joined tiles, tray (unfolds downward) · connectivity · volume · battery. -->
<div class="side-stack" title="Quick settings">
  {#if trayIcons.length}
    <div class="side" role="group" aria-label="Tray icons" bind:this={trayEl} onpointerenter={trayEnter} onpointerleave={trayLeave}>
      <button class="chev" class:open={expanded} title={expanded ? 'Hide tray icons' : `Show ${trayIcons.length} tray icons`}
              onpointerdown={(e) => e.stopPropagation()} onclick={(e) => { e.stopPropagation(); expanded = !expanded; }}>
        {#if expanded}<Close size={14} />{:else}<Open size={14} /><span class="count">{trayIcons.length}</span>{/if}
      </button>
      {#if expanded}
        <span class="grid" transition:slide={t}>{#each trayIcons as i (i.id)}<TrayIconButton icon={i} size={16} />{/each}</span>
      {/if}
    </div>
  {/if}
  <div class="side">
    <span class="grid">
      <DockMarks mode={settings.micOnDock} />
      {#if airplane}
        <Plane size={16} />
      {:else}
        {#if has('wifi')}{#if on('wifi')}<Wifi size={16} />{:else}<WifiOff size={16} class="dim" />{/if}{/if}
        {#if has('bluetooth')}{#if on('bluetooth')}<Bluetooth size={16} />{:else}<BluetoothOff size={16} class="dim" />{/if}{/if}
        {#if !radios.length}<Wifi size={16} />{/if}
      {/if}
      {#if lang}<b class="lang" title={lang.name}>{lang.code}</b>{/if}
    </span>
  </div>
  {#if settings.soundOnDock}<SoundPart {instance} {size} settings={soundOf(settings)} vertical />{/if}
  {#if bat.v}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="side" class:batfill={settings.batteryStyle === 'fill'} title="{bat.v.percent}% · {batWord}" role="button" tabindex="-1" onpointerdown={(e) => e.stopPropagation()}
         onclick={(e) => { e.stopPropagation(); ipc.popupOpen(BATTERY_PANEL + instance, e.currentTarget.getBoundingClientRect(), 320, 420); }}>
      {#if settings.batteryStyle === 'fill'}
      <BatteryFill b={bat.v} />
      {:else}
      <BatteryTall b={bat.v} />
      <span class="side-v">{bat.v.percent}<small>%</small></span>
      {#if bat.v.onAc}<span class="plugc" title={batWord}><Plug size={11} strokeWidth={2.4} /></span>{/if}
      {/if}
    </div>
  {/if}
</div>
{:else}
<!-- Three joined segments: tray · language · Wi-Fi · Bluetooth | volume | battery. -->
<div class="qswrap">
<div class="pill qs" title="Quick settings">
  {#if trayIcons.length}
    <span class="tray" role="group" aria-label="Tray icons" bind:this={trayEl} onpointerenter={trayEnter} onpointerleave={trayLeave}>
      <button class="chev" class:open={expanded} title={expanded ? 'Hide tray icons' : `Show ${trayIcons.length} tray icons`}
              onpointerdown={(e) => e.stopPropagation()} onclick={(e) => { e.stopPropagation(); expanded = !expanded; }}>
        {#if expanded}<Close size={14} />{:else}<Open size={14} /><span class="count">{trayIcons.length}</span>{/if}
      </button>
      {#if expanded}
        <span class="icons" transition:slide={t}>
          {#each trayIcons as i (i.id)}<TrayIconButton icon={i} size={16} />{/each}
        </span>
      {/if}
    </span>
    <span class="sep"></span>
  {/if}
  <DockMarks mode={settings.micOnDock} />
  {#if lang}<b class="lang" title={lang.name}>{lang.code}</b>{/if}
  {#if airplane}
    <Plane size={16} />
  {:else}
    {#if has('wifi')}{#if on('wifi')}<Wifi size={16} />{:else}<WifiOff size={16} class="dim" />{/if}{/if}
    {#if has('bluetooth')}{#if on('bluetooth')}<Bluetooth size={16} />{:else}<BluetoothOff size={16} class="dim" />{/if}{/if}
    {#if !radios.length}<Wifi size={16} />{/if}
  {/if}
</div>
{#if settings.soundOnDock}<SoundPart {instance} {size} settings={soundOf(settings)} vertical={false} />{/if}
{#if bat.v}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="pill bat" class:batfill={settings.batteryStyle === 'fill'} title="{bat.v.percent}% · {batWord}" role="button" tabindex="-1" onpointerdown={(e) => e.stopPropagation()}
       onclick={(e) => { e.stopPropagation(); ipc.popupOpen(BATTERY_PANEL + instance, e.currentTarget.getBoundingClientRect(), 320, 420); }}>
    {#if settings.batteryStyle === 'fill'}
      <!-- The whole battery part fills up to the charge level, % in the middle. -->
      <BatteryFill b={bat.v} />
    {:else}
      {#if settings.batteryStyle === 'liquid'}<LiquidBattery b={bat.v} />{:else}<BatteryIcon b={bat.v} size={16} />{/if}
      {#if bat.v.onAc}<span class="plug"><Plug size={12} strokeWidth={2.2} /></span>{/if}
    {/if}
  </div>
{/if}
</div>
{/if}

<style>
  .qs { gap: 6px; }
  .qswrap { display: flex; align-items: stretch; gap: 2px; height: 100%; }
  /* Inner corners of the joined segments are nearly square. */
  .qswrap > :global(.pill:not(:first-child)) { border-top-left-radius: calc(4px * var(--rs)); border-bottom-left-radius: calc(4px * var(--rs)); }
  .qswrap > :global(.pill:not(:last-child)) { border-top-right-radius: calc(4px * var(--rs)); border-bottom-right-radius: calc(4px * var(--rs)); }
  .tray, .icons { display: flex; align-items: center; gap: 0; }
  .chev { display: flex; align-items: center; gap: 2px; padding: 3px 4px; border-radius: calc(8px * var(--rs)); transition: background 0.15s; }
  .chev:hover, .chev.open { background: var(--pill-hover); }
  .lang { font-size: 11.5px; letter-spacing: 0.04em; min-width: 18px; text-align: center; }
  .bat { gap: 2px; padding: 0 9px; }
  /* Fill design: the part is the tank; the % sits in its middle. */
  .bat.batfill { position: relative; justify-content: center; min-width: 58px; padding: 0 10px; overflow: hidden; }
  .side.batfill { position: relative; overflow: hidden; min-height: 40px; justify-content: center; }
  .plug { display: inline-flex; color: #30d158; }
  .count { font-size: 10px; font-variant-numeric: tabular-nums; opacity: 0.75; }
  .icons { overflow: hidden; }
  .sep { width: 1px; align-self: stretch; margin: 9px 2px; background: var(--line-soft); }
  /* Left/right dock tiles */
  .grid { display: grid; grid-template-columns: repeat(2, auto); gap: 7px 8px; place-items: center; justify-content: center; }
  .plugc { display: inline-flex; color: #30d158; }
</style>
