<script lang="ts">
  // Real networks from Windows. Connecting a new secured network asks for its password,
  // which goes straight to Windows (GlassBar doesn't keep it).
  import { ChevronLeft, Eye, EyeOff, Lock, RefreshCw, Settings2, Wifi, WifiHigh, WifiLow, WifiZero } from '@lucide/svelte';
  import { events, ipc, type WifiNetwork } from '../../lib/ipc';

  let { back }: { back: () => void } = $props();
  let nets = $state<WifiNetwork[]>([]);
  let current = $state<string | null>(null);
  let loading = $state(true);
  let error = $state('');
  let open = $state<string | null>(null); // SSID with the password box open
  let password = $state('');
  let show = $state(false);
  let busy = $state<string | null>(null);
  let failed = $state<{ ssid: string; why: string } | null>(null);

  async function load(scan: boolean) {
    loading = true;
    try { const r = await ipc.wifiList(scan); nets = r.networks; current = r.current; error = ''; }
    catch (e) { error = String(e); }
    loading = false;
  }
  $effect(() => {
    load(false).then(() => load(true)); // show the last scan at once, then a fresh one
    const off = events.changed('wifi', () => load(false));
    return () => { off.then((f) => f()); };
  });

  async function connect(n: WifiNetwork, pw?: string) {
    if (n.secured && !n.saved && pw === undefined) { open = n.ssid; password = ''; failed = null; return; }
    busy = n.ssid; failed = null;
    try { await ipc.wifiConnect(n.ssid, pw ?? null); open = null; password = ''; }
    catch (e) { failed = { ssid: n.ssid, why: String(e) }; if (String(e).includes('password')) open = n.ssid; }
    busy = null;
    await load(false);
  }
  async function disconnect() {
    busy = current;
    try { await ipc.wifiDisconnect(); } catch (e) { error = String(e); }
    busy = null;
    await load(false);
  }
  const bars = (b: number) => (b >= 4 ? Wifi : b === 3 ? WifiHigh : b >= 1 ? WifiLow : WifiZero);
</script>

<div class="head">
  <button class="icon-btn" onclick={back} aria-label="Back"><ChevronLeft size={16} /></button>
  <h3>Wi-Fi</h3>
  <button class="icon-btn" class:spin={loading} onclick={() => load(true)} aria-label="Scan again"><RefreshCw size={14} /></button>
</div>

{#if error}
  <p class="err small">{error}</p>
  <button class="btn wide" onclick={() => ipc.openSettingsUri('ms-availablenetworks:')}>Open Windows network list</button>
{:else}
  <div class="list">
    {#each nets as n (n.ssid)}
      {@const Icon = bars(n.bars)}
      <div class="net" class:on={n.connected}>
        <button class="row" disabled={!!busy} onclick={() => (n.connected ? null : connect(n))}>
          <Icon size={16} />
          <span class="name"><span class="ellipsis">{n.ssid}</span>
            <span class="dim small">{n.connected ? 'Connected' : busy === n.ssid ? 'Connecting…' : n.saved ? `Saved · ${n.security}` : n.security}</span></span>
          {#if n.secured}<Lock size={12} class="dim" />{/if}
        </button>
        {#if n.connected}<button class="btn sm" disabled={!!busy} onclick={disconnect}>{busy === n.ssid ? '…' : 'Disconnect'}</button>{/if}
        {#if open === n.ssid}
          <form class="pw" onsubmit={(e) => { e.preventDefault(); connect(n, password); }}>
            <div class="pwbox">
              <input type={show ? 'text' : 'password'} autocomplete="off" placeholder="Network password" bind:value={password} />
              <button type="button" class="icon-btn" onclick={() => (show = !show)} aria-label={show ? 'Hide password' : 'Show password'}>
                {#if show}<EyeOff size={14} />{:else}<Eye size={14} />{/if}</button>
            </div>
            <div class="acts">
              <button type="button" class="btn sm" onclick={() => (open = null)}>Cancel</button>
              <button class="btn sm primary" disabled={!password || busy === n.ssid}>{busy === n.ssid ? 'Connecting…' : 'Connect'}</button>
            </div>
          </form>
        {/if}
        {#if failed?.ssid === n.ssid}
          <p class="err small">{failed.why} <button class="link" onclick={() => connect(n, open === n.ssid ? password : undefined)}>Retry</button></p>
        {/if}
      </div>
    {:else}
      <p class="dim small empty">{loading ? 'Scanning…' : 'No networks found.'}</p>
    {/each}
  </div>
{/if}
<button class="link foot" onclick={() => ipc.openSettingsUri('ms-settings:network-wifi')}><Settings2 size={12} /> Wi-Fi settings (hidden networks, enterprise sign-in)</button>

<style>
  .head { display: flex; align-items: center; gap: 6px; margin-bottom: 8px; }
  h3 { flex: 1; margin: 0; font-size: 15px; font-weight: 600; }
  .list { display: flex; flex-direction: column; gap: 4px; }
  .net { border-radius: calc(12px * var(--rs)); background: var(--pill); padding: 2px 6px 2px 2px; display: flex; flex-wrap: wrap; align-items: center; }
  .net.on { background: color-mix(in srgb, var(--accent) 22%, var(--pill)); }
  .row { flex: 1; min-width: 0; display: flex; align-items: center; gap: 10px; padding: 8px; text-align: left; border-radius: calc(10px * var(--rs)); }
  .name { flex: 1; min-width: 0; display: flex; flex-direction: column; font-size: 13px; }
  .pw { width: 100%; padding: 0 6px 8px; display: flex; flex-direction: column; gap: 6px; }
  .pwbox { display: flex; align-items: center; gap: 4px; }
  .pwbox input { flex: 1; }
  .acts { display: flex; justify-content: flex-end; gap: 6px; }
  .btn.sm { padding: 4px 10px; font-size: 12px; }
  .primary { background: var(--accent); color: #fff; }
  .wide { width: 100%; margin-top: 6px; }
  .small { font-size: 11px; }
  .err { color: #ff453a; width: 100%; margin: 0 8px 6px; }
  .link { color: var(--accent); font-size: 12px; }
  .foot { display: flex; align-items: center; gap: 4px; margin-top: 10px; }
  .empty { text-align: center; padding: 30px 0; }
  .spin :global(svg) { animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
