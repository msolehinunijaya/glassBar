<script lang="ts">
  import { Bluetooth, Cast, ChevronRight, Coffee, Headphones, Keyboard, Mic, MicOff, Monitor, Mouse, Plane, RadioTower, Settings, Share2, Smartphone, Sun, SunMoon, Wifi } from '@lucide/svelte';
  import AwakePanel from './AwakePanel.svelte';
  import MicPanel from './MicPanel.svelte';
  import { awakeText, levelTone, usingText } from './extras';
  import { live } from '../../lib/live.svelte';
  import { tickingNow } from '../../lib/now.svelte';
  import type { BtDevice, InUse, MicState } from '../../lib/ipc';
  import CastPanel from './CastPanel.svelte';
  import WifiPanel from './WifiPanel.svelte';
  import BtPanel from './BtPanel.svelte';
  import { events, ipc, type DisplayMode, type Hotspot, type RadioInfo } from '../../lib/ipc';
  import { connectedText } from './tileText';
  import type { PopupProps } from '../types';
  import type { QuickSettings, TileKey } from './index';
  import LangList from '../lang/LangList.svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import TraySection from './TraySection.svelte';

  let { instance, settings }: PopupProps<QuickSettings> = $props();
  let view = $state<'main' | 'wifi' | 'bluetooth' | 'cast' | 'awake' | 'mic'>('main');
  // Keep awake, the microphone and who uses it: live from Rust (events, no polling).
  const awake = live('awake', ipc.awakeGet, { on: false, until: null, screen: true });
  const awakeNow = tickingNow(() => awake.v.on && awake.v.until != null, 15_000);
  const mic = live<MicState | null>('mic', ipc.micState, null);
  const using = live<InUse>('privacy', ipc.privacyInUse, { mic: [], cam: [] });
  async function toggleAwake() {
    awake.v = awake.v.on ? await ipc.awakeOff() : await ipc.awakeSet((settings.awakeMinutes ?? 0) || null, settings.awakeScreen ?? true).catch((e) => { error = String(e); return awake.v; });
  }
  async function toggleMic() {
    if (!mic.v) return page('ms-settings:sound');
    const m = !mic.v.muted;
    mic.v = { ...mic.v, muted: m };
    await ipc.micSetMute(m).catch((e) => (error = String(e)));
  }
  // Bluetooth devices that report their battery (connected ones only).
  let btDevices = $state<BtDevice[]>([]);
  const withBattery = $derived(btDevices.filter((d) => d.connected && d.battery != null));
  const btIcon = (k: string) => (k === 'Audio' ? Headphones : k === 'Input' ? Keyboard : k === 'Phone' ? Smartphone : k === 'Bluetooth LE' ? Mouse : Bluetooth);
  let radios = $state<RadioInfo[]>([]);
  let mode = $state<DisplayMode | null>(null);
  /** Displays plugged in: with one, only "PC only" makes sense. */
  let displays = $state(2);
  let brightness = $state<number | null>(null);
  let error = $state('');
  let busy = $state(false);
  // Hotspot: real state from Windows, or the reason it can't be used (never a fake toggle).
  let hotspot = $state<Hotspot | null>(null);
  let hotspotWhy = $state('');
  const loadHotspot = () => ipc.hotspotGet().then((h) => { hotspot = h; hotspotWhy = ''; }).catch((e) => { hotspot = null; hotspotWhy = String(e); });

  // Night light and Nearby sharing have no public Windows API: the real state is read back,
  // and when that fails the tile opens the Windows page instead (null = couldn't read it).
  let night = $state<boolean | null>(null);
  let nightWhy = $state('');
  let nearby = $state<number | null>(null);
  const loadNight = () => ipc.nightLightGet().then((v) => { night = v; nightWhy = ''; }).catch((e) => { night = null; nightWhy = String(e); });
  const loadNearby = () => ipc.nearbyGet().then((v) => (nearby = v)).catch(() => (nearby = null));
  const NEARBY = ['Off', 'My devices', 'Everyone nearby'];
  const page = (uri: string) => ipc.openSettingsUri(uri).catch((e) => (error = String(e)));
  async function toggleNight() {
    if (night === null) return page('ms-settings:nightlight');
    await run(async () => { await ipc.nightLightSet(!night); });
    await loadNight();
  }
  async function toggleNearby() {
    if (nearby === null) return page('ms-settings:crossdevice');
    // Off <-> the on-mode used last (My devices only, unless it was Everyone nearby).
    const on = nearby > 0;
    if (on) updateWidgetSettings(instance, { nearbyMode: nearby as 1 | 2 });
    await run(() => ipc.nearbySet(on ? 0 : (settings.nearbyMode ?? 1)));
    await loadNearby();
  }
  const showTile = (k: TileKey) => !(settings.hiddenTiles ?? []).includes(k);

  const find = (k: RadioInfo['kind']) => radios.find((r) => r.kind === k);
  const airplane = $derived(radios.length > 0 && radios.every((r) => !r.on));
  const modes: [DisplayMode, string][] = [['internal', 'PC only'], ['clone', 'Duplicate'], ['extend', 'Extend'], ['external', 'Second only']];

  const loadRadios = () => ipc.radios().then((r) => { radios = r; }).catch((e) => (error = String(e)));
  // What Wi-Fi and Bluetooth are connected to, for the tiles (no scan: just Windows' current state).
  let wifiNet = $state<string | null>(null);
  let btNames = $state<string[]>([]);
  // When Windows refuses to say, the tile says why instead of a wrong "Not connected".
  let wifiNone = $state('Not connected');
  let btNone = $state('No devices connected');
  const loadLinks = () => {
    ipc.wifiList(false).then((r) => { wifiNet = r.current; wifiNone = 'Not connected'; })
      .catch((e) => { wifiNet = null; wifiNone = /blocked|Location/i.test(String(e)) ? 'Name hidden (Location off)' : 'Unknown'; });
    ipc.btList(false).then((d) => { btDevices = d; btNames = [...new Set(d.filter((x) => x.connected).map((x) => x.name))]; btNone = 'No devices connected'; })
      .catch(() => { btDevices = []; btNames = []; btNone = 'Unknown'; });
  };
  const linked = (kind: 'wifi' | 'bluetooth') =>
    kind === 'wifi' ? connectedText(wifiNet ? [wifiNet] : [], wifiNone) : connectedText(btNames, btNone);
  $effect(() => {
    loadLinks();
    const off = events.changed('wifi', loadLinks);
    return () => { off.then((f) => f()); };
  });
  $effect(() => {
    loadRadios();
    loadHotspot();
    loadNight();
    loadNearby();
    ipc.displayMode().then((m) => (mode = m));
    ipc.displayCount().then((n) => (displays = n)).catch(() => {});
    ipc.brightness().then((b) => (brightness = b));
  });

  async function run(f: () => Promise<unknown>) {
    busy = true;
    error = '';
    try { await f(); } catch (e) { error = String(e); }
    await loadRadios();
    await loadHotspot();
    loadLinks();
    busy = false;
  }

  let bTimer: ReturnType<typeof setTimeout>;
  function setBrightness(v: number) {
    brightness = v;
    clearTimeout(bTimer); // DDC/CI is slow; send only the settled value
    bTimer = setTimeout(() => ipc.setBrightness(v).catch((e) => (error = String(e))), 150);
  }
</script>

{#if view === 'wifi'}
  <WifiPanel back={() => (view = 'main')} />
{:else if view === 'cast'}
  <CastPanel back={() => (view = 'main')} />
{:else if view === 'awake'}
  <AwakePanel back={() => (view = 'main')} {instance} {settings} />
{:else if view === 'mic'}
  <MicPanel back={() => (view = 'main')} />
{:else if view === 'bluetooth'}
  <BtPanel back={() => (view = 'main')} radio={find('bluetooth')} toggle={() => run(() => ipc.setRadio('bluetooth', !find('bluetooth')!.on))} />
{:else}
<div class="qhead">
  <h3>Quick settings</h3>
  <button class="icon-btn" onclick={() => ipc.appLaunch('ms-settings:')} title="Open Windows Settings" aria-label="Open Windows Settings"><Settings size={16} /></button>
</div>
<!-- Split tiles: the left part toggles, the › part (after a hairline) opens more. -->
<div class="tiles">
  {#each [['wifi', 'Wi-Fi', Wifi], ['bluetooth', 'Bluetooth', Bluetooth]] as const as [kind, label, Icon] (kind)}
    {#if showTile(kind)}
    {@const r = find(kind)}
    {@const state = !r ? 'Not available' : r.on ? linked(kind) : 'Off'}
    <div class="tilewrap">
      <button class="tile" class:on={r?.on} disabled={!r || busy} onclick={() => run(() => ipc.setRadio(kind, !r!.on))}
              title={kind === 'bluetooth' && r?.on && btNames.length > 1 ? btNames.join(', ') : state}>
        <Icon size={18} />
        <span>{label}</span>
        <span class="state">{state}</span>
      </button>
      <button class="more" aria-label="{label} networks and devices" title={kind === 'wifi' ? 'Networks' : 'Devices'}
              onclick={() => (view = kind)}><ChevronRight size={16} /></button>
    </div>
    {/if}
  {/each}
  {#if showTile('airplane')}
  <button class="tile" class:on={airplane} disabled={!radios.length || busy} onclick={() => run(() => ipc.setAirplane(!airplane))}
          title="Turns all wireless radios off">
    <Plane size={18} />
    <span>Airplane</span>
    <span class="state">{airplane ? 'All radios off' : 'Off'}</span>
  </button>
  {/if}
  {#if showTile('hotspot')}
  <div class="tilewrap">
    <button class="tile" class:on={hotspot?.on} disabled={!hotspot || busy} onclick={() => run(() => ipc.hotspotSet(!hotspot!.on))}
            title={hotspotWhy || (hotspot?.ssid ? `Network: ${hotspot.ssid}` : 'Mobile hotspot')}>
      <RadioTower size={18} />
      <span>Hotspot</span>
      <span class="state">
        {#if !hotspot}Unavailable{:else if hotspot.on}On · {hotspot.clients}/{hotspot.maxClients} devices{:else}Off{/if}
      </span>
    </button>
    <button class="more" aria-label="Hotspot settings" title="Hotspot settings (name, password, band)" onclick={() => page('ms-settings:network-mobilehotspot')}><ChevronRight size={16} /></button>
  </div>
  {/if}
  {#if showTile('nearby')}
  <div class="tilewrap">
    <button class="tile" class:on={!!nearby} disabled={busy} onclick={toggleNearby}
            title={nearby === null ? 'Opens Nearby sharing in Settings' : 'Share files with PCs nearby. › for who can send to you'}>
      <Share2 size={18} />
      <span>Nearby sharing</span>
      <span class="state">{nearby === null ? 'Open settings' : NEARBY[nearby] ?? 'Off'}</span>
    </button>
    <button class="more" aria-label="Nearby sharing settings" title="Nearby sharing settings" onclick={() => page('ms-settings:crossdevice')}><ChevronRight size={16} /></button>
  </div>
  {/if}
  {#if showTile('nightlight')}
  <div class="tilewrap">
    <button class="tile" class:on={!!night} disabled={busy} onclick={toggleNight}
            title={night === null ? `${nightWhy || 'Night light'} · opens it in Settings` : 'Warmer colours to help you sleep. › for strength and schedule'}>
      <SunMoon size={18} />
      <span>Night light</span>
      <span class="state">{night === null ? 'Open settings' : night ? 'On' : 'Off'}</span>
    </button>
    <button class="more" aria-label="Night light settings" title="Night light settings (strength, schedule)" onclick={() => page('ms-settings:nightlight')}><ChevronRight size={16} /></button>
  </div>
  {/if}
  {#if showTile('awake')}
  <div class="tilewrap">
    <button class="tile" class:on={awake.v.on} onclick={toggleAwake} title="Stops the PC sleeping or dimming. › for how long">
      <Coffee size={18} />
      <span>Keep awake</span>
      <span class="state">{awakeText(awake.v, awakeNow.now)}</span>
    </button>
    <button class="more" aria-label="Keep awake options" title="How long, and the screen" onclick={() => (view = 'awake')}><ChevronRight size={16} /></button>
  </div>
  {/if}
  {#if showTile('mic')}
  {@const onAir = using.v.mic.length > 0}
  <div class="tilewrap">
    <button class="tile mic" class:muted={mic.v?.muted} class:on={!!mic.v && !mic.v.muted && onAir} disabled={busy} onclick={toggleMic}
            title={!mic.v ? 'No microphone found' : mic.v.muted ? 'Muted for every app. Click to unmute' : 'Click to mute for every app'}>
      {#if mic.v?.muted}<MicOff size={18} />{:else}<Mic size={18} />{/if}
      <span>Microphone{#if onAir && !mic.v?.muted}<i class="air" aria-hidden="true"></i>{/if}</span>
      <span class="state">{!mic.v ? 'Not found' : mic.v.muted ? 'Muted' : onAir ? `On · ${usingText(using.v.mic)}` : 'On'}</span>
    </button>
    <button class="more" aria-label="Microphone options" title="Level, device, apps using it" onclick={() => (view = 'mic')}><ChevronRight size={16} /></button>
  </div>
  {/if}
  {#if showTile('cast')}
  <div class="tilewrap">
    <button class="tile" onclick={() => (view = 'cast')} title="Show this screen on a TV or wireless display">
      <Cast size={18} />
      <span>Cast</span>
      <span class="state">Wireless display</span>
    </button>
    <button class="more" aria-label="Wireless displays" title="Wireless displays" onclick={() => (view = 'cast')}><ChevronRight size={16} /></button>
  </div>
  {/if}
</div>

{#if settings.btBattery !== false && withBattery.length}
  <h4 class="dim"><Bluetooth size={13} /> Devices</h4>
  <div class="devs">
    {#each withBattery as d (d.id)}
      {@const Icon = btIcon(d.kind)}
      <div class="dev" title="{d.name} · {d.battery}%">
        <Icon size={15} />
        <span class="ellipsis dn">{d.name}</span>
        <span class="bar {levelTone(d.battery!)}"><i style:width="{d.battery}%"></i></span>
        <span class="pc">{d.battery}%</span>
      </div>
    {/each}
  </div>
{/if}

<h4 class="dim"><Monitor size={13} /> Display</h4>
<div class="seg">
  {#each modes as [m, label] (m)}
    {@const off = displays < 2 && m !== 'internal'}
    <button class:sel={mode === m} disabled={off} title={off ? 'Connect a second display to use this' : ''}
            onclick={() => run(async () => { await ipc.setDisplayMode(m); mode = m; })}>{label}</button>
  {/each}
</div>
{#if displays < 2}<p class="dim note">No second display connected.</p>{/if}

<h4 class="dim"><Sun size={13} /> Brightness</h4>
{#if brightness !== null}
  <input class="slider" type="range" min="0" max="100" value={brightness} style:--fill="{brightness}%"
         oninput={(e) => setBrightness(+e.currentTarget.value)} aria-label="Brightness" />
{:else}
  <p class="dim small">This monitor does not support brightness control (DDC/CI off or not available).</p>
{/if}

{#if settings.showLanguage}
  <div class="lhead">
    <h4 class="dim">Keyboard language</h4>
    <label class="dim lsw">Show on dock
      <input type="checkbox" class="switch" checked={settings.langOnDock} onchange={(e) => updateWidgetSettings(instance, { langOnDock: e.currentTarget.checked })} />
    </label>
  </div>
  <LangList />
{/if}
{#if settings.showTray}
  <TraySection {instance} onDock={settings.trayOnDock} />
{/if}

{#if hotspotWhy}<p class="dim small">Hotspot: {hotspotWhy}</p>{/if}
{#if error}<p class="err">{error}</p>{/if}
{/if}

<style>
  h3 { margin: 0; font-size: 15px; font-weight: 600; }
  .qhead { display: flex; align-items: center; justify-content: space-between; margin: 0 0 10px; }
  .lhead { display: flex; align-items: center; justify-content: space-between; margin: 14px 2px 6px; }
  .lhead h4 { margin: 0; }
  .lsw { display: flex; align-items: center; gap: 6px; font-size: 11px; }
  h4 { display: flex; align-items: center; gap: 6px; margin: 14px 2px 6px; font-size: 12px; font-weight: 500; }
  .tiles { display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px; }
  .tile span { font-size: 12px; font-weight: 500; }
  .tile .state, .tile span { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 100%; }
  .tilewrap { position: relative; display: grid; }
  .tilewrap .tile { padding-right: 34px; }
  .more { position: absolute; top: 0; right: 0; bottom: 0; width: 30px; display: grid; place-items: center; border-radius: 0 14px 14px 0; opacity: 0.75; }
  .more:hover { opacity: 1; background: var(--pill-hover); }
  /* The chevron sits on the tile: on an "on" tile it takes the tile's text colour. */
  .tilewrap:has(.tile.on) .more { color: var(--on-accent); }
  /* The hairline that splits a tile into "toggle" and "more". */
  .more::before { content: ''; position: absolute; left: 0; top: 28%; bottom: 28%; width: 1px; background: currentColor; opacity: 0.22; }
  .more:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
  .more:active { transform: scale(0.94); }
  .tile .state { font-size: 10px; font-weight: 400; opacity: 0.75; }
  .seg { display: grid; grid-template-columns: repeat(4, 1fr); gap: 2px; padding: 3px; border-radius: calc(12px * var(--rs)); background: var(--pill); }
  .seg button { padding: 7px 2px; border-radius: calc(9px * var(--rs)); font-size: 11px; }
  .seg .sel { background: var(--accent); color: var(--on-accent); }
  .seg button:disabled { opacity: 0.35; cursor: default; }
  .note { font-size: 11px; margin: 6px 2px 0; }
  .small { font-size: 12px; }
  .err { color: #ff453a; font-size: 12px; }
  /* Muted mic: a red tile, so a muted mic is never missed. */
  .tile.mic.muted { background: color-mix(in srgb, #ff453a 22%, var(--pill)); }
  .tile.mic.muted :global(svg) { color: #ff453a; }
  .tilewrap:has(.tile.mic.muted) .more { color: inherit; }
  .air { display: inline-block; width: 6px; height: 6px; margin-left: 5px; vertical-align: middle; border-radius: 50%; background: #ff9f0a; box-shadow: 0 0 6px #ff9f0a; }
  .devs { display: flex; flex-direction: column; gap: 2px; }
  .dev { display: flex; align-items: center; gap: 10px; min-height: 40px; padding: 0 var(--s-3); border-radius: var(--r-inner); background: var(--bg-2); font-size: 13px; }
  .dn { flex: 1; min-width: 0; }
  .bar { width: 64px; height: 6px; border-radius: 3px; background: var(--track); overflow: hidden; flex: none; }
  .bar i { display: block; height: 100%; border-radius: 3px; background: #30d158; transition: width 0.4s var(--ease); }
  .bar.low i { background: #ff9f0a; }
  .bar.critical i { background: #ff453a; }
  .pc { width: 36px; text-align: right; font-size: 12px; font-variant-numeric: tabular-nums; }
</style>
