<script lang="ts">
  // Settings › Location: where GlassBar thinks you are, how it found that, how accurate it is.
  // Every location-based widget reads this one shared location.
  import { untrack } from 'svelte';
  import { Map as MapIcon, RefreshCw, Settings2, Trash2 } from '@lucide/svelte';
  import { store, update } from '../lib/config.svelte';
  import { ipc, type LocationMode, type Place } from '../lib/ipc';
  import { accuracyText, badge, coords, validLatLon } from '../lib/location';
  import { location as loc, refreshLocation, useLocation } from '../lib/location.svelte';

  useLocation();
  const cfg = $derived(store.config!.location);
  const s = $derived(loc.s);
  const b = $derived(badge(s));
  const p = $derived(s?.place ?? null);
  const pcTz = Intl.DateTimeFormat().resolvedOptions().timeZone;

  // A settings change is re-checked by Rust a moment later; show "Updating…" until it reports.
  let pending = $state(false);
  let seen = untrack(() => s);
  $effect(() => { if (s !== seen) { seen = s; pending = false; } });
  const setLoc = (patch: Partial<typeof cfg>) => { pending = true; update((c) => { c.location = { ...c.location, ...patch }; }); };

  const MODES: { id: LocationMode; title: string; how: string; acc: string }[] = [
    { id: 'auto', title: 'Automatic', how: 'Windows Location first. If that is off or denied, your internet address (IP).', acc: 'Depends on which one succeeds' },
    { id: 'windows', title: 'Windows Location (precise)', how: 'The Windows location service: GPS, Wi-Fi or cell network.', acc: 'Usually 10 m to 1 km' },
    { id: 'ip', title: 'IP address (approximate)', how: 'Guessed from your internet connection.', acc: 'City level, often 5–50 km; sometimes the wrong city or state' },
    { id: 'manual', title: 'Manual', how: 'A city you search for, or coordinates you type.', acc: 'Exactly what you set' },
  ];

  let query = $state('');
  let results = $state<Place[]>([]);
  let timer: ReturnType<typeof setTimeout>;
  function search(q: string) {
    query = q;
    clearTimeout(timer);
    timer = setTimeout(() => ipc.locationSearch(q).then((r) => (results = r)).catch(() => (results = [])), 350);
  }
  function pick(r: Place) {
    setLoc({ mode: 'manual', lat: r.lat, lon: r.lon, name: r.name, tz: r.tz ?? null });
    results = []; query = '';
  }

  let latText = $state(untrack(() => cfg.lat?.toString() ?? ''));
  let lonText = $state(untrack(() => cfg.lon?.toString() ?? ''));
  const latN = $derived(Number(latText));
  const lonN = $derived(Number(lonText));
  const coordsOk = $derived(latText.trim() !== '' && lonText.trim() !== '' && validLatLon(latN, lonN));
  const useCoords = () => coordsOk && setLoc({ mode: 'manual', lat: latN, lon: lonN, name: '', tz: null });

  let cleared = $state(false);
  async function clear() { pending = true; loc.s = await ipc.locationClear(); latText = lonText = ''; cleared = true; }

  const when = (ms: number) => new Date(ms).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });
</script>

<div class="group status">
  <div class="top">
    <span class="badge {b.kind}"><i></i>{b.label}</span>
    {#if pending || loc.busy}<span class="hint m0">Updating…</span>{/if}
  </div>
  {#if p}
    <b class="place">{p.name || coords(p)}</b>
    {#if !p.name && p.source === 'windows'}<span class="hint m0">No place name found for these coordinates (OpenStreetMap didn't answer).</span>{/if}
    {#if p.note}<span class="hint m0 warn">{p.note}.</span>{/if}
    <dl>
      <dt>Coordinates</dt><dd>{coords(p)}</dd>
      <dt>Accuracy</dt><dd>{p.source === 'manual' ? 'exactly as set' : accuracyText(p.accuracy)}{p.via ? ` (via ${p.via})` : ''}</dd>
      <dt>Time zone</dt><dd>{p.tz ?? `${pcTz} (this PC)`}</dd>
      <dt>Last updated</dt><dd>{s ? when(s.at) : '—'}</dd>
    </dl>
  {:else if s}
    <b class="place">No location</b>
    <span class="hint m0 warn">{s.reason}</span>
    {#if s.lastKnown}<span class="hint m0">Last known: {s.lastKnown.name || coords(s.lastKnown)} (not used by widgets while unavailable)</span>{/if}
    <span class="hint m0">Checked {when(s.at)}</span>
  {:else}
    <span class="hint m0">Checking…</span>
  {/if}
  <div class="acts">
    <button class="btn" disabled={loc.busy} onclick={refreshLocation}><RefreshCw size={14} /> Refresh now</button>
    {#if cfg.mode === 'auto' || cfg.mode === 'windows'}
      <button class="btn plain" onclick={() => ipc.openSettingsUri('ms-settings:privacy-location')}><Settings2 size={14} /> Open Windows location settings</button>
    {/if}
    {#if p}<button class="btn plain" onclick={() => ipc.locationOpenMap(p.lat, p.lon)}><MapIcon size={14} /> Open in Maps</button>{/if}
  </div>
</div>

<h2>Location source</h2>
<div class="group">
  {#each MODES as m (m.id)}
    <label class="mode">
      <input type="radio" name="locmode" checked={cfg.mode === m.id} onchange={() => setLoc({ mode: m.id })} />
      <span><b>{m.title}</b><span class="hint m0">{m.how} <em>{m.acc}.</em></span></span>
    </label>
  {/each}
  <p class="hint">Automatic and Windows Location re-check on start, every 30 minutes, after sleep and when the network changes. IP re-checks on start and when the network changes. Manual never changes by itself. Desktop PCs without Wi-Fi usually only get IP accuracy.</p>
</div>

{#if cfg.mode === 'manual'}
  <h2>Manual location</h2>
  <div class="group">
    <div class="field"><span>Current</span><b>{cfg.lat != null && cfg.lon != null ? cfg.name || `${cfg.lat.toFixed(4)}, ${cfg.lon.toFixed(4)}` : 'Not set'}</b></div>
    <input placeholder="Search a city…" value={query} oninput={(e) => search(e.currentTarget.value)} />
    {#each results as r (r.lat + ',' + r.lon)}
      <button class="res" onclick={() => pick(r)}>{r.name} <span class="hint m0">{coords(r)}{r.tz ? ` · ${r.tz}` : ''}</span></button>
    {/each}
    <details>
      <summary>Advanced: enter latitude and longitude</summary>
      <div class="ll">
        <input placeholder="Latitude (−90 to 90)" bind:value={latText} aria-label="Latitude" />
        <input placeholder="Longitude (−180 to 180)" bind:value={lonText} aria-label="Longitude" />
        <button class="btn" disabled={!coordsOk} onclick={useCoords}>Use</button>
      </div>
      {#if (latText || lonText) && !coordsOk}<p class="hint warn">Latitude must be between −90 and 90, longitude between −180 and 180.</p>{/if}
    </details>
  </div>
{/if}

<h2>Privacy</h2>
<div class="group">
  <p class="hint m0 pad">
    Coordinates (rounded to about 11 m) go only to the services that need them: Open-Meteo for weather and sunrise/sunset,
    waktusolat.app, JAKIM e-Solat or Aladhan for prayer times, and the DOE (APIMS) station list or Open-Meteo for air quality.
    For a Windows Location fix, OpenStreetMap (Nominatim) is asked for the town name, only when you have moved more than 1 km. IP mode asks ipwho.is (or ipapi.co) where your internet
    address is. City search sends only what you type to Open-Meteo. "Open in Maps" opens OpenStreetMap only when you click it.
    The last known location is kept on this PC only.
  </p>
  <div class="acts"><button class="btn plain" onclick={clear}><Trash2 size={14} /> Clear saved location</button>{#if cleared}<span class="hint m0">Cleared. Back to Automatic.</span>{/if}</div>
</div>

<style>
  .status { padding: 14px 16px; gap: 6px; }
  .top { display: flex; align-items: center; gap: 10px; }
  .badge { display: inline-flex; align-items: center; gap: 6px; font-size: 12px; font-weight: 600; padding: 3px 10px 3px 8px; border-radius: 999px; background: var(--pill); }
  .badge i { width: 9px; height: 9px; border-radius: 50%; background: #ff453a; }
  .badge.precise i { background: #30d158; }
  .badge.approx i { background: #ffd60a; }
  .badge.manual i { background: #0a84ff; }
  .place { font-size: 17px; font-weight: 600; }
  .m0 { margin: 0; }
  .pad { padding: 8px 0 0; }
  dl { display: grid; grid-template-columns: max-content 1fr; gap: 4px 16px; margin: 6px 0; font-size: 13px; }
  dt { color: var(--fg-dim); }
  dd { margin: 0; font-variant-numeric: tabular-nums; }
  .acts { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; padding: 6px 0 8px; }
  .btn.plain { background: var(--pill); color: var(--fg); }
  .mode { display: flex; align-items: flex-start; gap: 10px; padding: 8px 0; }
  .mode + .mode { border-top: 1px solid var(--line-soft); }
  .mode input { margin-top: 3px; }
  .mode > span { display: flex; flex-direction: column; gap: 2px; }
  .mode em { font-style: normal; color: var(--fg); }
  .res { display: flex; flex-direction: column; align-items: flex-start; text-align: left; padding: 6px 8px; border-radius: 8px; }
  .res:hover { background: var(--pill); }
  details { padding: 4px 0 8px; }
  summary { cursor: pointer; font-size: 13px; }
  .ll { display: flex; gap: 8px; margin-top: 8px; }
  .ll input { flex: 1; min-width: 0; }
</style>
