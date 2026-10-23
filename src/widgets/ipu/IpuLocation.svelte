<script lang="ts">
  // Choose where the air-quality reading comes from:
  //  · My location — nearest JAS/DOE station (or AQI (US) outside Malaysia)
  //  · Malaysia — browse state → station (real IPU from DOE's network)
  //  · Other country — search a city; shows AQI (US) from Open-Meteo, because IPU
  //    (DOE) only covers Malaysia. No state list is shown there: there is no source for it.
  import { Star, X } from '@lucide/svelte';
  import { ipc, type Place, type Station } from '../../lib/ipc';
  import { statesOf } from './browse';
  import type { Fav, IpuSettings } from './index';

  let { settings, save, stations }: { settings: IpuSettings; save: (p: Partial<IpuSettings>) => void; stations: Station[] } = $props();

  const states = $derived(statesOf(stations));
  const current = $derived(stations.find((s) => s.id === settings.station));
  let pickedState = $state('');
  const selState = $derived(pickedState || current?.state || states[0]?.[0] || '');
  const inState = $derived(states.find(([s]) => s === selState)?.[1] ?? []);

  let query = $state('');
  let results = $state<Place[]>([]);
  let timer: ReturnType<typeof setTimeout>;
  function search(q: string) {
    query = q;
    clearTimeout(timer);
    timer = setTimeout(() => ipc.locationSearch(q).then((r) => (results = r)), 300);
  }

  const setCountry = (country: IpuSettings['country']) => save({ country, ...(country === 'auto' ? { station: 'auto' } : {}) });
  function setState(s: string) {
    pickedState = s;
    const first = states.find(([x]) => x === s)?.[1][0];
    if (first) save({ country: 'MY', station: first.id });
  }
  function pickCity(p: Place) {
    save({ country: 'other', city: { name: p.name, lat: p.lat, lon: p.lon } });
    results = []; query = '';
  }

  // Favourites: the current selection, saved for one-click switching.
  const here = $derived<Fav | null>(
    settings.country === 'MY' && current ? { label: current.name.split(',')[0], country: 'MY', station: current.id }
    : settings.country === 'other' && settings.city ? { label: settings.city.name.split(',')[0], country: 'other', city: settings.city }
    : null);
  const same = (a: Fav, b: Fav) => a.country === b.country && a.station === b.station && a.city?.lat === b.city?.lat && a.city?.lon === b.city?.lon;
  const isFav = $derived(!!here && settings.favorites.some((f) => same(f, here!)));
  function toggleFav() {
    if (!here) return;
    save({ favorites: isFav ? settings.favorites.filter((f) => !same(f, here!)) : [...settings.favorites, here] });
  }
  const apply = (f: Fav) => save(f.country === 'MY' ? { country: 'MY', station: f.station } : { country: 'other', city: f.city });
</script>

<div class="loc">
  <div class="seg">
    <button class:sel={settings.country === 'auto'} onclick={() => setCountry('auto')}>My location</button>
    <button class:sel={settings.country === 'MY'} onclick={() => setCountry('MY')}>Malaysia</button>
    <button class:sel={settings.country === 'other'} onclick={() => setCountry('other')}>Other country</button>
  </div>

  {#if settings.country === 'MY'}
    {#if stations.length}
      <label class="row">State
        <select value={selState} onchange={(e) => setState(e.currentTarget.value)}>
          {#each states as [s] (s)}<option value={s}>{s}</option>{/each}
        </select>
      </label>
      <label class="row">Station
        <select value={settings.station} onchange={(e) => save({ country: 'MY', station: e.currentTarget.value })}>
          {#if !inState.some((s) => s.id === settings.station)}<option value={settings.station}>Choose a station…</option>{/if}
          {#each inState as s (s.id)}<option value={s.id}>{s.name.split(',')[0]} · {Math.round(s.api ?? 0)}</option>{/each}
        </select>
      </label>
    {:else}
      <p class="dim small">Station list unavailable (no connection to JAS/DOE right now).</p>
    {/if}
  {:else if settings.country === 'other'}
    <input class="search" placeholder="Search a city…" value={query} oninput={(e) => search(e.currentTarget.value)} />
    {#each results as r (r.lat + ',' + r.lon)}<button class="res" onclick={() => pickCity(r)}>{r.name}</button>{/each}
    {#if settings.city}<p class="small">Showing: <b>{settings.city.name}</b></p>{/if}
    <p class="dim small">IPU (JAS/DOE) only covers Malaysia. For other countries this shows the US Air Quality Index from Open-Meteo, which uses a different scale.</p>
  {:else}
    <p class="dim small">Nearest JAS/DOE station to you; outside Malaysia, the US Air Quality Index for your area.</p>
  {/if}

  {#if here || settings.favorites.length}
    <div class="favs">
      {#if here}
        <button class="fav-btn" class:on={isFav} onclick={toggleFav} aria-label={isFav ? 'Remove from favourites' : 'Save as favourite'}>
          <Star size={13} fill={isFav ? 'currentColor' : 'none'} /> {isFav ? 'Saved' : 'Save'}
        </button>
      {/if}
      {#each settings.favorites as f, i (i)}
        <span class="chip">
          <button onclick={() => apply(f)}>{f.label}</button>
          <button class="x" onclick={() => save({ favorites: settings.favorites.filter((_, j) => j !== i) })} aria-label="Remove {f.label}"><X size={10} /></button>
        </span>
      {/each}
    </div>
  {/if}
</div>

<style>
  .loc { display: flex; flex-direction: column; gap: 8px; }
  .seg { display: grid; grid-template-columns: repeat(3, 1fr); gap: 2px; padding: 2px; border-radius: 10px; background: var(--pill); }
  .seg button { padding: 6px 2px; border-radius: 8px; font-size: 11px; }
  .seg .sel { background: var(--accent); color: #fff; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 8px; font-size: 12px; }
  .row select { flex: 1; max-width: 200px; padding: 5px 8px; border-radius: 8px; border: 1px solid var(--line-soft); background: var(--pill); }
  .search { padding: 6px 10px; border-radius: 9px; border: 1px solid var(--line-soft); background: var(--pill); outline: none; }
  .res { text-align: left; padding: 5px 8px; border-radius: 8px; font-size: 12px; }
  .res:hover { background: var(--pill-hover); }
  .small { font-size: 11px; margin: 0; }
  .favs { display: flex; flex-wrap: wrap; gap: 5px; }
  .fav-btn { display: inline-flex; align-items: center; gap: 4px; font-size: 11px; padding: 3px 9px; border-radius: 10px; background: var(--pill); }
  .fav-btn.on { color: #ffb340; }
  .chip { display: inline-flex; align-items: center; border-radius: 10px; background: var(--pill); font-size: 11px; }
  .chip button { padding: 3px 4px 3px 9px; }
  .chip .x { padding: 3px 6px 3px 2px; opacity: 0.6; display: grid; place-items: center; }
</style>
