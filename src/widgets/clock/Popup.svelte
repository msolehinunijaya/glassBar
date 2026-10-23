<script lang="ts">
  import { ArrowDown, ArrowUp, ChevronLeft, ChevronRight, Moon, Plus, Settings2, Sun, Sunrise, Sunset, X } from '@lucide/svelte';
  import FormatOptions from './FormatOptions.svelte';
  import Time from './ClockTime.svelte';
  import { date, timeText } from './prefs.svelte';
  import { dayState, type DayState } from './solar';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc, type Place } from '../../lib/ipc';
  import type { PopupProps } from '../types';
  import type { ClockSettings } from './index';
  import { ticker } from './time.svelte';
  import { cityOf, diffLabel, localOffset, offsetMinutes, zonesOf, type Zone } from './zones';

  let { instance, settings }: PopupProps<ClockSettings> = $props();
  // Every second only when seconds are shown somewhere in the panel; otherwise each minute.
  const clock = ticker(() => settings.showSeconds || settings.worldSeconds);
  let showFormat = $state(false);
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
  const day = (tz: string) => date(clock.now, settings, tz);
  // Day/night per city (real sunrise/sunset when the city has coordinates), on the clock's tick.
  const icons: Record<DayState, typeof Sun> = { day: Sun, night: Moon, sunrise: Sunrise, sunset: Sunset };
  function light(z: Zone) {
    const info = dayState(clock.now.getTime(), z.tz, z.lat != null && z.lon != null ? { lat: z.lat, lon: z.lon } : undefined);
    const name = { day: 'Day', night: 'Night', sunrise: 'Sunrise', sunset: 'Sunset' }[info.state];
    const next = info.state === 'day' || info.state === 'sunset'
      ? (info.nextSet ? ` · sunset at ${timeText(new Date(info.nextSet), settings, false, z.tz)}` : '')
      : (info.nextRise ? ` · sunrise at ${timeText(new Date(info.nextRise), settings, false, z.tz)}` : '');
    const how = z.lat == null ? ' (approximate: 6 AM–6 PM)' : '';
    return { ...info, icon: icons[info.state], tip: name + next + how };
  }

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
  <div class="big"><Time now={clock.now} {settings} /></div>
  <div class="dim">{date(clock.now, settings)}</div>
  <button class="icon-btn gear" class:on={showFormat} onclick={() => (showFormat = !showFormat)} aria-label="Time and date format"><Settings2 size={16} /></button>
</div>
{#if showFormat}
  <div class="fmt"><FormatOptions {settings} save={(p) => updateWidgetSettings(instance, p)} /></div>
{/if}

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
        <button onclick={() => add({ tz: p.tz!, label: p.name.split(',')[0], lat: p.lat, lon: p.lon })}>{p.name} <span class="dim">· {p.tz}</span></button>
      {/each}
      {#each zoneResults as z (z)}
        <button onclick={() => add({ tz: z, label: cityOf(z) })}>{cityOf(z)} <span class="dim">· {z}</span></button>
      {/each}
    </div>
  {/if}
  {#each zones as z, i (z.tz + z.label)}
    {@const l = settings.dayNight ? light(z) : null}
    <div class="zrow" class:night={l?.state === 'night'} class:dayrow={l?.state === 'day'} class:twi={l?.state === 'sunrise' || l?.state === 'sunset'}>
      {#if l}<span class="dn" title={l.tip} aria-label={l.tip}><l.icon size={16} /></span>{/if}
      <div class="zcol">
        <b class="ellipsis">{z.label}</b>
        <span class="dim small">{day(z.tz)} · {diffLabel(localOffset(clock.now), offsetMinutes(z.tz, clock.now))}</span>
      </div>
      <span class="ztime"><Time now={clock.now} {settings} seconds={settings.worldSeconds} tz={z.tz} /></span>
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
  .head { position: relative; padding: 2px 4px 10px; }
  .gear { position: absolute; top: 0; right: 0; }
  .gear.on { color: var(--accent); }
  .fmt { margin-bottom: 10px; }
  .dn { display: grid; place-items: center; width: 26px; height: 26px; border-radius: 13px; flex: none; }
  .dayrow .dn { color: #ffb340; }
  .night .dn { color: #9fb4ff; }
  .twi .dn { color: #ff8a65; }
  /* a light tint by state: warm by day, deep blue at night */
  .zrow.dayrow { background: color-mix(in srgb, #ffcc4d 8%, transparent); }
  .zrow.night { background: color-mix(in srgb, #1e3a8a 22%, transparent); }
  .zrow.twi { background: color-mix(in srgb, #ff8a65 10%, transparent); }
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
