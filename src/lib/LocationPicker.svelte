<script lang="ts">
  // Shared location setting (Weather, Waktu Solat, IPU): auto, or a searched city.
  import { store, update } from './config.svelte';
  import { ipc, type Place } from './ipc';

  let query = $state('');
  let results = $state<Place[]>([]);
  let detected = $state<Place | null>(null);
  let failed = $state(false);
  let timer: ReturnType<typeof setTimeout>;
  const loc = $derived(store.config!.location);

  $effect(() => {
    if (loc.auto) ipc.locationGet(false).then((p) => { detected = p; failed = !p; }).catch(() => (failed = true));
  });

  function search(q: string) {
    query = q;
    clearTimeout(timer);
    timer = setTimeout(() => ipc.locationSearch(q).then((r) => (results = r)), 350);
  }
  function pick(p: Place) {
    update((c) => { c.location = { auto: false, lat: p.lat, lon: p.lon, name: p.name }; });
    results = []; query = '';
  }
</script>

<label><input type="checkbox" checked={loc.auto} onchange={(e) => update((c) => (c.location.auto = e.currentTarget.checked))} /> Detect my location automatically</label>
{#if loc.auto}
  <p class="hint">
    {#if detected}Detected: {detected.name} ({detected.source === 'ip' ? 'approximate, from your IP' : 'Windows location'})
    {:else if failed}Couldn't detect your location. Search for your city below.
    {:else}Detecting…{/if}
  </p>
{:else}
  <p class="hint">Using: {loc.name || 'not set'}</p>
{/if}
<input placeholder="Search a city…" value={query} oninput={(e) => search(e.currentTarget.value)} />
{#each results as r (r.lat + ',' + r.lon)}
  <button class="res" onclick={() => pick(r)}>{r.name}</button>
{/each}

<style>
  .hint { font-size: 12px; color: var(--fg-dim); margin: 0; }
  .res { text-align: left; padding: 6px 8px; border-radius: 8px; }
  .res:hover { background: var(--pill); }
</style>
