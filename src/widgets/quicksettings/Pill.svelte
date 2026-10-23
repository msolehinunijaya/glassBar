<script lang="ts">
  import { Bluetooth, BluetoothOff, ChevronDown, ChevronLeft, ChevronRight, ChevronUp, Plane, Wifi, WifiOff } from '@lucide/svelte';
  import { slide } from 'svelte/transition';
  import { ipc, type RadioInfo } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import type { QuickSettings } from './index';
  import TrayIconButton from '../tray/TrayIconButton.svelte';
  import { liveTray } from '../tray/icons.svelte';

  let { settings, vertical }: PillProps<QuickSettings> = $props();
  // Tray icons on the dock (switch in the panel), collapsed behind a chevron by default.
  // Click the chevron to show them all; a click anywhere else on the dock (another widget,
  // the bar itself) or leaving GlassBar folds them away again.
  const tray = liveTray();
  const trayIcons = $derived(settings.trayOnDock ? [...tray.v].sort((a, b) => Number(b.promoted) - Number(a.promoted)) : []);
  let expanded = $state(false);
  let trayEl = $state<HTMLElement>();
  $effect(() => {
    if (!expanded) return;
    const outside = (e: PointerEvent) => { if (!trayEl?.contains(e.target as Node)) expanded = false; };
    const leave = () => (expanded = false);
    addEventListener('pointerdown', outside, true);
    addEventListener('blur', leave);
    return () => { removeEventListener('pointerdown', outside, true); removeEventListener('blur', leave); };
  });
  // The chevron points the way the icons unfold (towards the screen centre on side docks).
  const Open = $derived(vertical ? ChevronDown : ChevronLeft);
  const Close = $derived(vertical ? ChevronUp : ChevronRight);
  const t = $derived({ axis: (vertical ? 'y' : 'x') as 'x' | 'y', duration: 220 });
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
  {#if airplane}
    <Plane size={16} />
  {:else}
    {#if has('wifi')}{#if on('wifi')}<Wifi size={16} />{:else}<WifiOff size={16} class="dim" />{/if}{/if}
    {#if has('bluetooth')}{#if on('bluetooth')}<Bluetooth size={16} />{:else}<BluetoothOff size={16} class="dim" />{/if}{/if}
    {#if !radios.length}<Wifi size={16} />{/if}
  {/if}
</div>

<style>
  .qs { gap: 6px; }
  .tray, .icons { display: flex; align-items: center; gap: 0; }
  .vertical .tray, .vertical .icons { flex-direction: column; }
  .chev { display: flex; align-items: center; gap: 2px; padding: 3px 4px; border-radius: 8px; transition: background 0.15s; }
  .chev:hover, .chev.open { background: var(--pill-hover); }
  .count { font-size: 10px; font-variant-numeric: tabular-nums; opacity: 0.75; }
  .icons { overflow: hidden; }
  .sep { width: 1px; align-self: stretch; margin: 9px 2px; background: var(--line-soft); }
  .vertical .sep { width: auto; height: 1px; margin: 2px 8px; }
</style>
