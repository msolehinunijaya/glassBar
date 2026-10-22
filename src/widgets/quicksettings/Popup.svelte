<script lang="ts">
  import { Bluetooth, Monitor, Plane, Sun, Wifi } from '@lucide/svelte';
  import { ipc, type DisplayMode, type RadioInfo } from '../../lib/ipc';
  import type { PopupProps } from '../types';

  let {}: PopupProps = $props();
  let radios = $state<RadioInfo[]>([]);
  let mode = $state<DisplayMode | null>(null);
  let brightness = $state<number | null>(null);
  let error = $state('');
  let busy = $state(false);

  const find = (k: RadioInfo['kind']) => radios.find((r) => r.kind === k);
  const airplane = $derived(radios.length > 0 && radios.every((r) => !r.on));
  const modes: [DisplayMode, string][] = [['internal', 'PC only'], ['clone', 'Duplicate'], ['extend', 'Extend'], ['external', 'Second only']];

  const loadRadios = () => ipc.radios().then((r) => { radios = r; }).catch((e) => (error = String(e)));
  $effect(() => {
    loadRadios();
    ipc.displayMode().then((m) => (mode = m));
    ipc.brightness().then((b) => (brightness = b));
  });

  async function run(f: () => Promise<unknown>) {
    busy = true;
    error = '';
    try { await f(); } catch (e) { error = String(e); }
    await loadRadios();
    busy = false;
  }

  let bTimer: ReturnType<typeof setTimeout>;
  function setBrightness(v: number) {
    brightness = v;
    clearTimeout(bTimer); // DDC/CI is slow; send only the settled value
    bTimer = setTimeout(() => ipc.setBrightness(v).catch((e) => (error = String(e))), 150);
  }
</script>

<h3>Quick settings</h3>
<div class="tiles">
  {#each [['wifi', 'Wi-Fi', Wifi], ['bluetooth', 'Bluetooth', Bluetooth]] as const as [kind, label, Icon] (kind)}
    {@const r = find(kind)}
    <button class="tile" class:on={r?.on} disabled={!r || busy} onclick={() => run(() => ipc.setRadio(kind, !r!.on))}>
      <Icon size={18} />
      <span>{label}</span>
      <span class="state">{!r ? 'Not available' : r.on ? 'On' : 'Off'}</span>
    </button>
  {/each}
  <button class="tile" class:on={airplane} disabled={!radios.length || busy} onclick={() => run(() => ipc.setAirplane(!airplane))}
          title="Turns all wireless radios off">
    <Plane size={18} />
    <span>Airplane</span>
    <span class="state">{airplane ? 'All radios off' : 'Off'}</span>
  </button>
</div>

<h4 class="dim"><Monitor size={13} /> Display</h4>
<div class="seg">
  {#each modes as [m, label] (m)}
    <button class:sel={mode === m} onclick={() => run(async () => { await ipc.setDisplayMode(m); mode = m; })}>{label}</button>
  {/each}
</div>

<h4 class="dim"><Sun size={13} /> Brightness</h4>
{#if brightness !== null}
  <input class="slider" type="range" min="0" max="100" value={brightness} style:--fill="{brightness}%"
         oninput={(e) => setBrightness(+e.currentTarget.value)} aria-label="Brightness" />
{:else}
  <p class="dim small">This monitor does not support brightness control (DDC/CI off or not available).</p>
{/if}

{#if error}<p class="err">{error}</p>{/if}

<style>
  h3 { margin: 0 0 10px; font-size: 15px; font-weight: 600; }
  h4 { display: flex; align-items: center; gap: 6px; margin: 14px 2px 6px; font-size: 12px; font-weight: 500; }
  .tiles { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
  .tile span { font-size: 12px; font-weight: 500; }
  .tile .state { font-size: 10px; font-weight: 400; opacity: 0.75; }
  .seg { display: grid; grid-template-columns: repeat(4, 1fr); gap: 2px; padding: 3px; border-radius: 12px; background: var(--pill); }
  .seg button { padding: 7px 2px; border-radius: 9px; font-size: 11px; }
  .seg .sel { background: var(--accent); color: #fff; }
  .small { font-size: 12px; }
  .err { color: #ff453a; font-size: 12px; }
</style>
