<script lang="ts">
  import { Check, Speaker, Volume2, VolumeX } from '@lucide/svelte';
  import { ipc, type AudioSession } from '../../lib/ipc';
  import { layoutOf, type SoundSettings } from './settings';
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import { liveAudio, pct, volumeIcon } from './audio.svelte';
  import { volumeBlip } from '../../lib/sound';

  // Opened from the volume segment of Quick Settings; its options are saved there.
  let { settings, save }: { settings: SoundSettings; save: (p: Partial<SoundSettings>) => void } = $props();
  const audio = liveAudio();
  const Icon = $derived(volumeIcon(audio.v));
  let sessions = $state<AudioSession[]>([]);
  let icons = $state<Record<string, string | null>>({});
  let error = $state('');
  const requested = new Set<string>();

  // Per-app list refreshes while the panel is open (apps start/stop playing).
  $effect(() => {
    const load = () => ipc.audioSessions().then((s) => {
      sessions = s;
      for (const x of s) if (x.exe && !requested.has(x.exe)) { requested.add(x.exe); ipc.appIcon(x.exe).then((i) => (icons[x.exe] = i)); }
    }).catch((e) => (error = String(e)));
    load();
    const t = setInterval(load, 2000);
    return () => clearInterval(t);
  });

  function setMaster(v: number, final = false) {
    if (!audio.v) return;
    audio.v = { ...audio.v, volume: v };
    ipc.audioSetVolume(v);
    volumeBlip(final);
  }
  function toggleMute() {
    if (!audio.v) return;
    audio.v = { ...audio.v, muted: !audio.v.muted };
    ipc.audioSetMute(audio.v.muted);
  }
  async function setDevice(id: string) {
    try { await ipc.audioSetDefault(id); error = ''; audio.v = await ipc.audioState(); }
    catch (e) { error = `Could not switch device: ${e}`; }
  }
  function setApp(s: AudioSession, volume: number) { s.volume = volume; ipc.audioSetSessionVolume(s.pid, volume); }
  function muteApp(s: AudioSession) { s.muted = !s.muted; ipc.audioSetSessionMute(s.pid, s.muted); }
</script>

<div class="head"><h3>Sound</h3></div>
{#if audio.v}
  <div class="master">
    <button class="icon-btn" onclick={toggleMute} aria-label={audio.v.muted ? 'Unmute' : 'Mute'}><Icon size={18} /></button>
    <input class="slider" type="range" min="0" max="1" step="0.01" value={audio.v.volume}
           style:--fill={pct(audio.v.muted ? 0 : audio.v.volume)}
           oninput={(e) => setMaster(+e.currentTarget.value)} onchange={(e) => setMaster(+e.currentTarget.value, true)} aria-label="Master volume" />
    <span class="num">{pct(audio.v.volume)}</span>
  </div>

  <h4 class="dim">Output</h4>
  <div class="card list">
    {#each audio.v.devices as d (d.id)}
      <button class="row" onclick={() => setDevice(d.id)}>
        <Speaker size={16} /><span class="ellipsis">{d.name}</span>
        {#if d.isDefault}<Check size={16} color="var(--accent)" />{/if}
      </button>
    {/each}
  </div>
{:else}
  <p class="dim">No audio device found.</p>
{/if}

{#if sessions.length}
  <h4 class="dim">Apps</h4>
  <div class="card list">
    {#each sessions as s (s.pid)}
      <div class="app">
        {#if icons[s.exe]}<img src={icons[s.exe]} alt="" width="20" height="20" />{:else}<Volume2 size={18} />{/if}
        <div class="col">
          <span class="ellipsis small">{s.name}</span>
          <input class="slider thin" type="range" min="0" max="1" step="0.01" value={s.volume}
                 style:--fill={pct(s.muted ? 0 : s.volume)} oninput={(e) => setApp(s, +e.currentTarget.value)} aria-label="{s.name} volume" />
        </div>
        <button class="icon-btn" onclick={() => muteApp(s)} aria-label="Mute {s.name}">
          {#if s.muted}<VolumeX size={16} />{:else}<Volume2 size={16} />{/if}
        </button>
      </div>
    {/each}
  </div>
{/if}
{#if error}<p class="err">{error}</p>{/if}

<div class="opts">
  <OptionsPopover title="On the bar" values={{ ...settings, layout: layoutOf(settings) }}
    save={(p) => save({ ...p, showSlider: false })} options={[
      { type: 'choice', key: 'layout', label: 'Volume', choices: [{ value: 'icon', label: 'Icon', hint: 'Speaker icon (click it to mute)' }, { value: 'horizontal', label: 'Horizontal', hint: 'Icon and percentage; a slider opens when you hover' }] },
      { type: 'toggle', key: 'wheel', label: 'Mouse wheel changes volume', hint: 'Off: scrolling over it never changes the volume' },
  ]} />
  {#if layoutOf(settings) === 'horizontal'}
    <label class="slen">Slider length <input type="range" min="60" max="240" step="10" value={settings.sliderWidth}
      oninput={(e) => save({ sliderWidth: +e.currentTarget.value })} /> <span class="dim">{settings.sliderWidth}px</span></label>
  {/if}
</div>

<style>
  h3 { margin: 0; font-size: 15px; font-weight: 600; }
  .head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; }
  .opts { margin-top: 12px; }
  .slen { display: flex; align-items: center; gap: 8px; font-size: 12px; margin-top: 8px; }
  .slen input { flex: 1; }
  h4 { margin: 14px 2px 6px; font-size: 12px; font-weight: 500; }
  .master { display: flex; align-items: center; gap: 8px; }
  .num { font-variant-numeric: tabular-nums; width: 4ch; text-align: right; }
  .list { padding: 4px; display: flex; flex-direction: column; }
  .row { display: flex; align-items: center; gap: 10px; padding: 8px 10px; border-radius: calc(12px * var(--rs)); text-align: left; }
  .row span { flex: 1; }
  .row:hover { background: var(--pill-hover); }
  .app { display: flex; align-items: center; gap: 10px; padding: 6px 8px; }
  .col { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .small { font-size: 12px; }
  .thin { height: 14px; }
  .thin::-webkit-slider-runnable-track { height: 14px !important; }
  .thin::-webkit-slider-thumb { width: 14px !important; height: 14px !important; }
  .err { color: #ff453a; font-size: 12px; }
</style>
