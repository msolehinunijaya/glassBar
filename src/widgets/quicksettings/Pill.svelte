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
  import SoundPart from '../sound/SoundPart.svelte';
  import { BATTERY_PANEL, soundOf } from '../sound/settings';

  let { instance, size, settings, vertical }: PillProps<QuickSettings> = $props();
  // Tray icons on the dock (switch in the panel), collapsed behind a chevron by default.
  // Click the chevron to show them all; a click anywhere else on the dock (another widget,
  // the bar itself) folds them away again. Not on losing focus: right-clicking a tray icon
  // opens that app's own menu, which takes focus, and the icons must stay put for it.
  const tray = liveTray();
  const trayIcons = $derived(settings.trayOnDock ? [...tray.v].sort((a, b) => Number(b.promoted) - Number(a.promoted)) : []);
  let expanded = $state(false);
  let trayEl = $state<HTMLElement>();
  $effect(() => {
    if (!expanded) return;
    const outside = (e: PointerEvent) => { if (!trayEl?.contains(e.target as Node)) expanded = false; };
    addEventListener('pointerdown', outside, true);
    return () => removeEventListener('pointerdown', outside, true);
  });
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

<!-- Three joined segments: tray · language · Wi-Fi · Bluetooth | volume | battery. -->
<div class="qswrap" class:vertical>
<div class="pill qs" class:vertical title="Quick settings">
  {#if trayIcons.length}
    <span class="tray" bind:this={trayEl}>
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
  {#if lang}<b class="lang" title={lang.name}>{lang.code}</b>{/if}
  {#if airplane}
    <Plane size={16} />
  {:else}
    {#if has('wifi')}{#if on('wifi')}<Wifi size={16} />{:else}<WifiOff size={16} class="dim" />{/if}{/if}
    {#if has('bluetooth')}{#if on('bluetooth')}<Bluetooth size={16} />{:else}<BluetoothOff size={16} class="dim" />{/if}{/if}
    {#if !radios.length}<Wifi size={16} />{/if}
  {/if}
</div>
{#if settings.soundOnDock}<SoundPart {instance} {size} settings={soundOf(settings)} {vertical} />{/if}
{#if bat.v}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="pill bat" class:vertical title="{bat.v.percent}% · {batWord}" role="button" tabindex="-1" onpointerdown={(e) => e.stopPropagation()}
       onclick={(e) => { e.stopPropagation(); ipc.popupOpen(BATTERY_PANEL + instance, e.currentTarget.getBoundingClientRect(), 320, 420); }}>
    {#if settings.batteryStyle === 'liquid'}<LiquidBattery b={bat.v} />{:else}<BatteryIcon b={bat.v} size={16} />{/if}
    {#if bat.v.onAc}<span class="plug"><Plug size={12} strokeWidth={2.2} /></span>{/if}
  </div>
{/if}
</div>

<style>
  .qs { gap: 6px; }
  .qswrap { display: flex; align-items: stretch; gap: 2px; height: 100%; }
  .qswrap.vertical { flex-direction: column; height: auto; }
  /* Inner corners of the joined segments are nearly square. */
  .qswrap:not(.vertical) > :global(.pill:not(:first-child)) { border-top-left-radius: 4px; border-bottom-left-radius: 4px; }
  .qswrap:not(.vertical) > :global(.pill:not(:last-child)) { border-top-right-radius: 4px; border-bottom-right-radius: 4px; }
  .qswrap.vertical > :global(.pill:not(:first-child)) { border-top-left-radius: 4px; border-top-right-radius: 4px; }
  .qswrap.vertical > :global(.pill:not(:last-child)) { border-bottom-left-radius: 4px; border-bottom-right-radius: 4px; }
  .tray, .icons { display: flex; align-items: center; gap: 0; }
  .vertical .tray, .vertical .icons { flex-direction: column; }
  .chev { display: flex; align-items: center; gap: 2px; padding: 3px 4px; border-radius: 8px; transition: background 0.15s; }
  .chev:hover, .chev.open { background: var(--pill-hover); }
  .lang { font-size: 11.5px; letter-spacing: 0.04em; min-width: 18px; text-align: center; }
  .bat { gap: 2px; padding: 0 9px; }
  .bat.vertical { flex-direction: column; padding: 6px 0; }
  .plug { display: inline-flex; color: #30d158; }
  .count { font-size: 10px; font-variant-numeric: tabular-nums; opacity: 0.75; }
  .icons { overflow: hidden; }
  .sep { width: 1px; align-self: stretch; margin: 9px 2px; background: var(--line-soft); }
  .vertical .sep { width: auto; height: 1px; margin: 2px 8px; }
</style>
