<script lang="ts">
  // Paired and nearby Bluetooth devices from Windows. Pairing runs Windows' own flow
  // (its PIN/confirm dialog). Connecting a paired device has no public Windows API, so
  // that goes to Windows Bluetooth settings rather than a button that only pretends.
  import { Bluetooth, ChevronLeft, Headphones, Keyboard, RefreshCw, Settings2, Smartphone, Laptop } from '@lucide/svelte';
  import { ipc, type BtDevice, type RadioInfo } from '../../lib/ipc';

  let { back, radio, toggle }: { back: () => void; radio: RadioInfo | undefined; toggle: () => Promise<void> } = $props();
  let devices = $state<BtDevice[]>([]);
  let loading = $state(false);
  let error = $state('');
  let busy = $state<string | null>(null);
  let msg = $state<{ id: string; text: string; ok: boolean } | null>(null);

  async function load(discover: boolean) {
    if (!radio?.on) { devices = []; return; }
    loading = true;
    try { devices = await ipc.btList(discover); error = ''; } catch (e) { error = String(e); }
    loading = false;
  }
  $effect(() => { if (radio?.on) load(false).then(() => load(true)); });

  async function act(d: BtDevice, f: () => Promise<void>, ok: string) {
    busy = d.id; msg = null;
    try { await f(); msg = { id: d.id, text: ok, ok: true }; } catch (e) { msg = { id: d.id, text: String(e), ok: false }; }
    busy = null;
    await load(false);
  }
  const icon = (k: string) => (k === 'Audio' ? Headphones : k === 'Input' ? Keyboard : k === 'Phone' ? Smartphone : k === 'Computer' ? Laptop : Bluetooth);
  const paired = $derived(devices.filter((d) => d.paired));
  const nearby = $derived(devices.filter((d) => !d.paired));
</script>

<div class="head">
  <button class="icon-btn" onclick={back} aria-label="Back"><ChevronLeft size={16} /></button>
  <h3>Bluetooth</h3>
  {#if radio}
    <input type="checkbox" class="switch" checked={radio.on} onchange={toggle} aria-label="Bluetooth on or off" />
  {/if}
  <button class="icon-btn" class:spin={loading} disabled={!radio?.on} onclick={() => load(true)} aria-label="Search again"><RefreshCw size={14} /></button>
</div>

{#if !radio}
  <p class="dim small">This PC has no Bluetooth adapter, or its driver isn't installed.</p>
{:else if !radio.on}
  <p class="dim small">Bluetooth is off.</p>
{:else if error}
  <p class="err small">{error}</p>
{:else}
  <div class="list">
    {#if paired.length}<h4 class="dim">Your devices</h4>{/if}
    {#each paired as d (d.id)}
      {@const Icon = icon(d.kind)}
      <div class="dev" class:on={d.connected}>
        <Icon size={16} />
        <span class="name"><span class="ellipsis">{d.name}</span><span class="dim small">{d.connected ? 'Connected' : 'Paired'} · {d.kind}</span></span>
        <button class="btn sm" onclick={() => ipc.openSettingsUri('ms-settings:bluetooth')} title="Windows has no public API for apps to connect or disconnect a device">{d.connected ? 'Disconnect…' : 'Connect…'}</button>
        <button class="btn sm" disabled={busy === d.id} onclick={() => act(d, () => ipc.btUnpair(d.id), 'Removed')}>{busy === d.id ? '…' : 'Remove'}</button>
        {#if msg?.id === d.id}<p class="small" class:err={!msg.ok}>{msg.text}</p>{/if}
      </div>
    {/each}
    <h4 class="dim">Nearby {#if loading}<span class="small">· searching…</span>{/if}</h4>
    {#each nearby as d (d.id)}
      {@const Icon = icon(d.kind)}
      <div class="dev">
        <Icon size={16} />
        <span class="name"><span class="ellipsis">{d.name}</span><span class="dim small">{d.kind}</span></span>
        <button class="btn sm primary" disabled={!!busy} onclick={() => act(d, () => ipc.btPair(d.id), 'Paired')}>{busy === d.id ? 'Pairing…' : 'Pair'}</button>
        {#if msg?.id === d.id}<p class="small" class:err={!msg.ok}>{msg.text}{#if !msg.ok} <button class="link" onclick={() => act(d, () => ipc.btPair(d.id), 'Paired')}>Retry</button>{/if}</p>{/if}
      </div>
    {:else}
      {#if !loading}<p class="dim small">No discoverable devices. Put the device in pairing mode, then search again.</p>{/if}
    {/each}
  </div>
{/if}
<button class="link foot" onclick={() => ipc.openSettingsUri('ms-settings:bluetooth')}><Settings2 size={12} /> Bluetooth settings</button>

<style>
  .head { display: flex; align-items: center; gap: 6px; margin-bottom: 8px; }
  h3 { flex: 1; margin: 0; font-size: 15px; font-weight: 600; }
  h4 { margin: 8px 2px 2px; font-size: 11px; font-weight: 500; }
  .list { display: flex; flex-direction: column; gap: 4px; }
  .dev { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; padding: 8px 10px; border-radius: calc(12px * var(--rs)); background: var(--pill); }
  .dev.on { background: color-mix(in srgb, var(--accent) 22%, var(--pill)); }
  .name { flex: 1; min-width: 0; display: flex; flex-direction: column; font-size: 13px; }
  .dev p { width: 100%; margin: 0; }
  .btn.sm { padding: 4px 8px; font-size: 11px; }
  .primary { background: var(--accent); color: var(--on-accent); }
  .small { font-size: 11px; }
  .err { color: #ff453a; }
  .link { color: var(--accent); font-size: 12px; }
  .foot { display: flex; align-items: center; gap: 4px; margin-top: 10px; }
  .spin :global(svg) { animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
