<script lang="ts">
  // The one shared location (Weather, sunrise/sunset, Waktu Solat, IPU). Says honestly
  // where it came from: Windows location (with its accuracy), IP (approximate) or manual.
  import { store, update } from './config.svelte';
  import { ipc, type Place } from './ipc';

  let query = $state('');
  let results = $state<Place[]>([]);
  let detected = $state<Place | null>(null);
  let status = $state<'idle' | 'busy' | 'failed'>('idle');
  let timer: ReturnType<typeof setTimeout>;
  const loc = $derived(store.config!.location);

  async function detect(force: boolean) {
    status = 'busy';
    try { detected = await ipc.locationGet(force); status = detected ? 'idle' : 'failed'; } catch { status = 'failed'; }
  }
  $effect(() => { if (loc.auto) { loc.useDevice; detect(false); } });

  function search(q: string) {
    query = q;
    clearTimeout(timer);
    timer = setTimeout(() => ipc.locationSearch(q).then((r) => (results = r)), 350);
  }
  function pick(p: Place) {
    update((c) => { c.location = { ...c.location, auto: false, lat: p.lat, lon: p.lon, name: p.name }; });
    results = []; query = '';
  }
  function reset() {
    update((c) => { c.location = { auto: true, useDevice: true, lat: null, lon: null, name: '' }; });
    detect(true);
  }
  const how = (p: Place) => p.source === 'windows'
    ? `Windows location${p.note ? ` via ${p.note}` : ''}${p.accuracy ? `, within about ${p.accuracy >= 1000 ? `${(p.accuracy / 1000).toFixed(1)} km` : `${p.accuracy} m`}` : ''}`
    : p.source === 'ip' ? 'Approximate (from your internet address, often off by several km)' : 'Chosen by you';
</script>

<label><input type="checkbox" checked={loc.auto} onchange={(e) => update((c) => (c.location.auto = e.currentTarget.checked))} /> Detect my location automatically</label>
{#if loc.auto}
  <label class="sub"><input type="checkbox" checked={loc.useDevice} onchange={(e) => update((c) => (c.location.useDevice = e.currentTarget.checked))} />
    Use device location (Windows location service) — off = approximate IP location only</label>
  <div class="cur">
    {#if status === 'busy'}<span class="hint">Detecting…</span>
    {:else if detected}
      <span><b>{detected.name}</b></span>
      <span class="hint">{how(detected)}</span>
      {#if detected.source === 'ip' && detected.note}<span class="hint warn">{detected.note}</span>{/if}
    {:else}<span class="hint warn">Couldn't detect your location. Search for your city below.</span>{/if}
  </div>
{:else}
  <div class="cur"><span><b>{loc.name || 'Not set'}</b></span><span class="hint">Chosen by you</span></div>
{/if}
<div class="acts">
  {#if loc.auto}<button class="btn" disabled={status === 'busy'} onclick={() => detect(true)}>Refresh location</button>{/if}
  <button class="btn" onclick={reset}>Reset</button>
</div>
<input placeholder="Search a city to set it manually…" value={query} oninput={(e) => search(e.currentTarget.value)} />
{#each results as r (r.lat + ',' + r.lon)}
  <button class="res" onclick={() => pick(r)}>{r.name}</button>
{/each}
<p class="hint">Only a rounded position (~100 m) is used, and it is sent only to the weather, prayer-time and air-quality services that need it.</p>

<style>
  .hint { font-size: 12px; color: var(--fg-dim); margin: 0; }
  .warn { color: #ff9f0a; }
  .sub { padding-left: 22px; font-size: 12px; }
  .cur { display: flex; flex-direction: column; gap: 2px; padding: 8px 10px; border-radius: 10px; background: var(--pill); margin: 4px 0; }
  .acts { display: flex; gap: 6px; margin-bottom: 6px; }
  .res { text-align: left; padding: 6px 8px; border-radius: 8px; }
  .res:hover { background: var(--pill); }
</style>
