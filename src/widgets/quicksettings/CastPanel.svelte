<script lang="ts">
  // Cast: wireless displays (Miracast) nearby, and whether this PC is casting now. Windows
  // only lets its own Cast panel start or end a cast, and only that panel has the switch for
  // touch, mouse and keyboard input from the display -- so Connect / Disconnect open it,
  // and say so, rather than pretend.
  import { Cast, ChevronLeft, ExternalLink, MonitorSmartphone, RefreshCw } from '@lucide/svelte';
  import { ipc, type CastState } from '../../lib/ipc';

  let { back }: { back: () => void } = $props();
  const CAST_UI = 'ms-settings-connectabledevices:devicediscovery';
  let found = $state<CastState | null>(null);
  let loading = $state(false);
  let error = $state('');

  async function load() {
    loading = true;
    error = '';
    try { found = await ipc.castGet(); } catch (e) { error = String(e); }
    loading = false;
  }
  $effect(() => { load(); });
  const openCast = () => ipc.openSettingsUri(CAST_UI).catch((e) => (error = String(e)));
</script>

<div class="head">
  <button class="icon-btn" onclick={back} aria-label="Back"><ChevronLeft size={16} /></button>
  <h3>Cast</h3>
  <button class="icon-btn" class:spin={loading} disabled={loading} onclick={load} aria-label="Search again"><RefreshCw size={14} /></button>
</div>

{#if found?.casting}
  <div class="dev on">
    <Cast size={16} />
    <span class="name"><span>Casting to a wireless display</span><span class="dim small">Touch and keyboard input: in Windows' Cast panel</span></span>
    <button class="btn sm" onclick={openCast} title="Windows only lets its own Cast panel end a cast">Disconnect…</button>
  </div>
{/if}

{#if error}
  <p class="err small">{error}</p>
{:else if !found && loading}
  <p class="dim small searching">Looking for wireless displays…</p>
{:else if found && !found.displays.length}
  <p class="dim small">No wireless displays found. Turn the TV or display's screen mirroring on, and keep it near this PC.</p>
{:else if found}
  <div class="list">
    <h4 class="dim">Nearby</h4>
    {#each found.displays as d (d.id)}
      <div class="dev">
        <MonitorSmartphone size={16} />
        <span class="name ellipsis">{d.name}</span>
        <button class="btn sm" onclick={openCast} title="Windows only lets its own Cast panel start a cast">Connect…</button>
      </div>
    {/each}
  </div>
{/if}

<div class="foot">
  <button class="link" onclick={openCast}>Open Windows Cast <ExternalLink size={11} /></button>
</div>
<p class="dim small note">Windows lets only its own Cast panel connect, disconnect, or allow touch from the display. GlassBar finds the displays and opens that panel for you.</p>

<style>
  .head { display: flex; align-items: center; gap: 6px; margin-bottom: 8px; }
  h3 { flex: 1; margin: 0; font-size: 15px; font-weight: 600; }
  h4 { margin: 8px 2px 2px; font-size: 11px; font-weight: 500; }
  .list { display: flex; flex-direction: column; gap: 4px; }
  .dev { display: flex; align-items: center; gap: 8px; padding: 8px 10px; border-radius: calc(12px * var(--rs)); background: var(--pill); margin-bottom: 4px; }
  .dev.on { background: color-mix(in srgb, var(--accent) 22%, var(--pill)); }
  .name { flex: 1; min-width: 0; display: flex; flex-direction: column; font-size: 13px; }
  .btn.sm { padding: 4px 8px; font-size: 11px; }
  .small { font-size: 11px; }
  .err { color: #ff453a; }
  .link { display: inline-flex; align-items: center; gap: 4px; color: var(--accent); font-size: 12px; }
  .foot { display: flex; align-items: center; gap: 4px; margin-top: 10px; }
  .note { margin: 6px 2px 0; line-height: 1.4; }
  .searching { animation: pulse 1.4s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: 0.5; } }
  .spin :global(svg) { animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  @media (prefers-reduced-motion: reduce) { .searching, .spin :global(svg) { animation: none; } }
</style>
