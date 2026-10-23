<script lang="ts">
  import { Bluetooth, BluetoothOff, Plane, Wifi, WifiOff } from '@lucide/svelte';
  import { ipc, type RadioInfo } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import type { QuickSettings } from './index';
  import TrayIconButton from '../tray/TrayIconButton.svelte';
  import { liveTray } from '../tray/icons.svelte';

  let { settings, vertical }: PillProps<QuickSettings> = $props();
  // Tray icons on the dock (switch in the panel): Windows' "always show" ones first.
  const tray = liveTray();
  const MAX = 6;
  const trayIcons = $derived(settings.trayOnDock ? [...tray.v].sort((a, b) => Number(b.promoted) - Number(a.promoted)) : []);
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
    <span class="tray">
      {#each trayIcons.slice(0, MAX) as i (i.id)}<TrayIconButton icon={i} size={16} />{/each}
      {#if trayIcons.length > MAX}<span class="more dim">+{trayIcons.length - MAX}</span>{/if}
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
  .tray { display: flex; align-items: center; gap: 0; }
  .vertical .tray { flex-direction: column; }
  .more { font-size: 10px; padding: 0 2px; }
  .sep { width: 1px; align-self: stretch; margin: 9px 2px; background: var(--line-soft); }
  .vertical .sep { width: auto; height: 1px; margin: 2px 8px; }
</style>
