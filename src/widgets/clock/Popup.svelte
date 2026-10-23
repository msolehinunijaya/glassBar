<script lang="ts">
  import { ArrowDown, ArrowUp, ChevronLeft, ChevronRight, Plus, X } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc, type Place } from '../../lib/ipc';
  import type { PopupProps } from '../types';
  import type { ClockSettings } from './index';
  import { fmtTime, ticker } from './time.svelte';
  import { cityOf, diffLabel, localOffset, offsetMinutes, zonesOf, type Zone } from './zones';

  let { instance, settings }: PopupProps<ClockSettings> = $props();
  const clock = ticker(() => true);
  const today = $derived(clock.now);
  let month = $state(new Date(new Date().getFullYear(), new Date().getMonth(), 1));
  const zones = $derived(zonesOf(settings));
  const save = (z: Zone[]) => updateWidgetSettings(instance, { zones: z, secondTz: '' });

  // ---- world clocks ----
  let adding = $state(false);
  let query = $state('');
  let cityResults = $state<Place[]>([]);
  let timer: ReturnType<typeof setTimeout>;
  const allZones = Intl.supportedValuesOf('timeZone');
  // Matching time-zone ids (e.g. "tokyo", "london") plus real city search with each city's zone.
  const zoneResults = $derived(query.trim().length < 2 ? [] :
    allZones.filter((z) => z.toLowerCase().replace(/_/g, ' ').includes(query.trim().toLowerCase())).slice(0, 5));
  function search(q: string) {
    query = q;
    clearTimeout(timer);
    if (q.trim().length < 2) { cityResults = []; return; }
    timer = setTimeout(() => ipc.locationSearch(q).then((r) => (cityResults = r.filter((p) => p.tz))), 300);
  }
  function add(z: Zone) {
    if (!zones.some((x) => x.tz === z.tz && x.label === z.label)) save([...zones, z]);
    adding = false; query = ''; cityResults = [];
  }
  const move = (i: number, d: number) => { const z = [...zones]; [z[i], z[i + d]] = [z[i + d], z[i]]; save(z); };
  const day = (tz: string) => new Intl.DateTimeFormat(undefined, { weekday: 'short', day: 'numeric', month: 'short', timeZone: tz }).format(clock.now);

  // ---- calendar (Monday-first) ----
  const cells = $derived.by(() => {
    const lead = (month.getDay() + 6) % 7;
    const days = new Date(month.getFullYear(), month.getMonth() + 1, 0).getDate();
    return [...Array(lead).fill(0), ...Array.from({ length: days }, (_, i) => i + 1)];
  });
  const isToday = (d: number) => d === today.getDate() && month.getMonth() === today.getMonth() && month.getFullYear() === today.getFullYear();
  const shift = (n: number) => (month = new Date(month.getFullYear(), month.getMonth() + n, 1));
  const weekdays = Array.from({ length: 7 }, (_, i) => new Intl.DateTimeFormat(undefined, { weekday: 'narrow' }).format(new Date(2024, 0, 1 + i)));
</script>

<div class="head">
  <div class="big">{fmtTime(clock.now, settings.hour12, true)}</div>
  <div class="dim">{new Intl.DateTimeFormat(undefined, { dateStyle: 'full' }).format(clock.now)}</div>
</div>

<div class="world">
  <div class="wtitle">
    <span class="dim">World clocks</span>
    <button class="icon-btn add" onclick={() => (adding = !adding)} aria-label="Add a city"><Plus size={15} /></button>
  </div>
  {#if adding}
    <!-- svelte-ignore a11y_autofocus -->
    <input class="search" placeholder="Search a city or time zone…" value={query} oninput={(e) => search(e.currentTarget.value)} autofocus />
    <div class="results">
      {#each cityResults as p (p.name + p.tz)}
        <button onclick={() => add({ tz: p.tz!, label: p.name.split(',')[0] })}>{p.name} <span class="dim">· {p.tz}</span></button>
      {/each}
      {#each zoneResults as z (z)}
        <button onclick={() => add({ tz: z, label: cityOf(z) })}>{cityOf(z)} <span class="dim">· {z}</span></button>
      {/each}
    </div>
  {/if}
  {#each zones as z, i (z.tz + z.label)}
    <div class="zrow">
      <div class="zcol">
        <b class="ellipsis">{z.label}</b>
        <span class="dim small">{day(z.tz)} · {diffLabel(localOffset(clock.now), offsetMinutes(z.tz, clock.now))}</span>
      </div>
      <span class="ztime">{fmtTime(clock.now, settings.hour12, false, z.tz)}</span>
      <div class="ztools">
        <button disabled={i === 0} onclick={() => move(i, -1)} aria-label="Move up"><ArrowUp size={12} /></button>
        <button disabled={i === zones.length - 1} onclick={() => move(i, 1)} aria-label="Move down"><ArrowDown size={12} /></button>
        <button onclick={() => save(zones.filter((_, j) => j !== i))} aria-label="Remove {z.label}"><X size={12} /></button>
      </div>
    </div>
  {:else}
    {#if !adding}<p class="dim small empty">Add cities with + to compare times.</p>{/if}
  {/each}
</div>

<div class="card cal">
  <div class="nav">
    <button class="icon-btn" onclick={() => shift(-1)} aria-label="Previous month"><ChevronLeft size={16} /></button>
    <button class="title" onclick={() => (month = new Date(today.getFullYear(), today.getMonth(), 1))}>
      {new Intl.DateTimeFormat(undefined, { month: 'long', year: 'numeric' }).format(month)}
    </button>
    <button class="icon-btn" onclick={() => shift(1)} aria-label="Next month"><ChevronRight size={16} /></button>
  </div>
  <div class="grid">
    {#each weekdays as w, i (i)}<span class="wd dim">{w}</span>{/each}
    {#each cells as d, i (i)}<span class="day" class:today={d && isToday(d)}>{d || ''}</span>{/each}
  </div>
</div>

<style>
  .head { padding: 2px 4px 10px; }
  .big { font-size: 30px; font-weight: 600; font-variant-numeric: tabular-nums; line-height: 1.1; }
  .world { margin-bottom: 10px; }
  .wtitle { display: flex; align-items: center; justify-content: space-between; font-size: 12px; padding: 0 4px; }
  .add { width: 26px; height: 26px; }
  .search { width: 100%; padding: 7px 10px; border-radius: 10px; border: 1px solid var(--line-soft); background: var(--pill); margin: 4px 0; outline: none; }
  .results { display: flex; flex-direction: column; }
  .results button { text-align: left; padding: 5px 8px; border-radius: 8px; font-size: 12px; }
  .results button:hover { background: var(--pill-hover); }
  .zrow { display: flex; align-items: center; gap: 8px; padding: 6px 6px; border-radius: 10px; }
  .zrow:hover { background: var(--pill); }
  .zcol { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .zcol b { font-weight: 500; }
  .ztime { font-size: 18px; font-weight: 600; font-variant-numeric: tabular-nums; }
  .ztools { display: flex; opacity: 0; transition: opacity 0.2s; }
  .zrow:hover .ztools, .zrow:focus-within .ztools { opacity: 1; }
  .ztools button { width: 20px; height: 20px; display: grid; place-items: center; border-radius: 50%; }
  .ztools button:hover { background: var(--pill-hover); }
  .ztools button:disabled { opacity: 0.3; }
  .small { font-size: 11px; }
  .empty { padding: 4px 6px; margin: 0; }
  .cal { padding: 10px; }
  .nav { display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px; }
  .title { font-weight: 600; padding: 4px 10px; border-radius: 10px; }
  .title:hover { background: var(--pill-hover); }
  .grid { display: grid; grid-template-columns: repeat(7, 1fr); gap: 2px; text-align: center; }
  .wd { font-size: 11px; padding: 3px 0; }
  .day { padding: 5px 0; border-radius: 50%; font-variant-numeric: tabular-nums; font-size: 12px; }
  .today { background: var(--accent); color: #fff; font-weight: 600; }
</style>
