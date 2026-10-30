<script lang="ts">
  // Microphone: mute, level, which recording device, and who is using the microphone or
  // camera right now (Windows' privacy store, the data behind its own taskbar indicator).
  import { Camera, Check, ChevronLeft, Mic, MicOff, Settings2 } from '@lucide/svelte';
  import { events, ipc, type InUse, type MicState } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';

  let { back }: { back: () => void } = $props();
  let mic = $state<MicState | null>(null);
  let error = $state('');
  const load = () => ipc.micState().then((m) => { mic = m; error = ''; }).catch((e) => { mic = null; error = String(e); });
  $effect(() => {
    load();
    const off = events.changed('mic', load);
    return () => { off.then((f) => f()); };
  });
  const using = live<InUse>('privacy', ipc.privacyInUse, { mic: [], cam: [] });

  async function mute() {
    if (!mic) return;
    mic.muted = !mic.muted;
    await ipc.micSetMute(mic.muted).catch((e) => (error = String(e)));
  }
  let vTimer: ReturnType<typeof setTimeout>;
  function volume(v: number) {
    if (!mic) return;
    mic.volume = v / 100;
    clearTimeout(vTimer);
    vTimer = setTimeout(() => ipc.micSetVolume(v / 100).catch((e) => (error = String(e))), 80);
  }
  const since = (ms: number) => (ms ? `since ${new Intl.DateTimeFormat(undefined, { hour: 'numeric', minute: '2-digit' }).format(ms)}` : '');
</script>

<div class="head">
  <button class="icon-btn" onclick={back} aria-label="Back"><ChevronLeft size={16} /></button>
  <h3>Microphone</h3>
</div>

{#if mic}
  <button class="hero" class:muted={mic.muted} onclick={mute} aria-pressed={mic.muted}>
    <span class="ic">{#if mic.muted}<MicOff size={20} />{:else}<Mic size={20} />{/if}</span>
    <span class="col"><b>{mic.muted ? 'Muted' : 'On'}</b><span class="dim">{mic.muted ? 'Apps hear nothing. Click to unmute.' : 'Click to mute for every app'}</span></span>
  </button>

  <label class="vol">
    <span class="dim">Level</span>
    <input class="slider" type="range" min="0" max="100" value={Math.round(mic.volume * 100)} style:--fill="{Math.round(mic.volume * 100)}%"
           disabled={mic.muted} oninput={(e) => volume(+e.currentTarget.value)} aria-label="Microphone level" />
    <span class="pct">{Math.round(mic.volume * 100)}%</span>
  </label>

  {#if mic.devices.length > 1}
    <h4 class="dim">Input device</h4>
    <div class="list">
      {#each mic.devices as d (d.id)}
        <button class="row" class:sel={d.isDefault} onclick={() => ipc.micSetDefault(d.id).then(load).catch((e) => (error = String(e)))}>
          <span class="ellipsis">{d.name}</span>{#if d.isDefault}<Check size={16} />{/if}
        </button>
      {/each}
    </div>
  {/if}
{:else if error}
  <p class="dim small">{error}</p>
{:else}
  <p class="dim small">Reading the microphone…</p>
{/if}

<h4 class="dim">In use now</h4>
{#if !using.v.mic.length && !using.v.cam.length}
  <p class="dim small none">No app is using your microphone or camera.</p>
{:else}
  <div class="list">
    {#each using.v.mic as a (a.name + a.since)}
      <div class="row app"><span class="dot mic"></span><Mic size={14} /><span class="ellipsis grow" title={a.exe}>{a.name}</span><span class="dim small">{since(a.since)}</span></div>
    {/each}
    {#each using.v.cam as a (a.name + a.since)}
      <div class="row app"><span class="dot cam"></span><Camera size={14} /><span class="ellipsis grow" title={a.exe}>{a.name}</span><span class="dim small">{since(a.since)}</span></div>
    {/each}
  </div>
{/if}
{#if error && mic}<p class="err">{error}</p>{/if}
<button class="link foot" onclick={() => ipc.openSettingsUri('ms-settings:privacy-microphone')}><Settings2 size={12} /> Microphone privacy settings</button>

<style>
  .head { display: flex; align-items: center; gap: 6px; margin-bottom: 10px; }
  h3 { flex: 1; margin: 0; font-size: 15px; font-weight: 600; }
  h4 { margin: 14px 2px 6px; font-size: 12px; font-weight: 500; }
  .hero { display: flex; align-items: center; gap: 12px; width: 100%; padding: 12px; border-radius: var(--r-inner); background: var(--bg-2); text-align: left;
          transition: background 0.2s var(--ease), transform 0.15s var(--ease); }
  .hero:hover { background: var(--pill-hover); }
  .hero:active { transform: scale(0.99); }
  .hero:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .ic { display: grid; place-items: center; width: 36px; height: 36px; border-radius: 50%; background: var(--accent); color: var(--on-accent); flex: none; }
  .hero.muted { background: color-mix(in srgb, #ff453a 16%, var(--bg-2)); }
  .hero.muted .ic { background: #ff453a; color: #fff; }
  .col { display: flex; flex-direction: column; line-height: 1.3; }
  .col b { font-size: 13px; }
  .col .dim { font-size: 12px; }
  .vol { display: flex; align-items: center; gap: 10px; margin: 12px 2px 0; font-size: 12px; }
  .vol .slider { flex: 1; }
  .vol .slider:disabled { opacity: 0.4; }
  .pct { min-width: 36px; text-align: right; font-variant-numeric: tabular-nums; }
  .list { display: flex; flex-direction: column; gap: 2px; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 8px; min-height: 40px; padding: 0 var(--s-3); border-radius: var(--r-inner); font-size: 13px; text-align: left; }
  button.row:hover { background: var(--bg-2); }
  .row.sel { background: var(--bg-2); box-shadow: inset 3px 0 0 var(--accent); font-weight: 600; }
  .row.app { justify-content: flex-start; }
  .grow { flex: 1; min-width: 0; }
  .dot { width: 8px; height: 8px; border-radius: 50%; flex: none; }
  .dot.mic { background: #ff9f0a; box-shadow: 0 0 6px #ff9f0a; }
  .dot.cam { background: #30d158; box-shadow: 0 0 6px #30d158; }
  .small { font-size: 11.5px; }
  .none { margin: 0 2px; }
  .link { display: flex; align-items: center; gap: 4px; color: var(--accent); font-size: 12px; }
  .foot { margin-top: 12px; }
  .err { color: #ff453a; font-size: 12px; }
</style>
