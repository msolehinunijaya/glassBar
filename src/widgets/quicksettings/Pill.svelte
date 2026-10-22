<script lang="ts">
  import { Bluetooth, BluetoothOff, Plane, Wifi, WifiOff } from '@lucide/svelte';
  import { ipc, type RadioInfo } from '../../lib/ipc';
  import type { PillProps } from '../types';

  let {}: PillProps = $props();
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

<div class="pill" title="Quick settings">
  {#if airplane}
    <Plane size={16} />
  {:else}
    {#if has('wifi')}{#if on('wifi')}<Wifi size={16} />{:else}<WifiOff size={16} class="dim" />{/if}{/if}
    {#if has('bluetooth')}{#if on('bluetooth')}<Bluetooth size={16} />{:else}<BluetoothOff size={16} class="dim" />{/if}{/if}
    {#if !radios.length}<Wifi size={16} />{/if}
  {/if}
</div>
