<script lang="ts">
  import { MapPin } from '@lucide/svelte';
  import type { PopupProps } from '../types';
  import { BANDS, bandOf } from './bands';
  import type { IpuSettings } from './index';
  import { label, liveIpu } from './ipu.svelte';

  let { settings }: PopupProps<IpuSettings> = $props();
  const ipu = liveIpu(() => settings.station);
  const time = (ms: number) => new Intl.DateTimeFormat(undefined, { weekday: 'short', hour: '2-digit', minute: '2-digit' }).format(ms);
  const km = (d: number | null) => (d == null ? '' : d < 1 ? '<1 km' : `${Math.round(d)} km`);
</script>

{#if ipu.r}
  {@const r = ipu.r}
  {@const b = BANDS[r.band]}
  <div class="hero" style:--c={b.color}>
    <span class="lbl">{label(r)}</span>
    <span class="big">{Math.round(r.value)}</span>
    <span class="band">{b.name}</span>
  </div>
  {#if r.station}
    <div class="row"><MapPin size={13} /> <b>{r.station.name}</b> <span class="dim">{km(r.distanceKm)}</span></div>
    <div class="dim small">Reading {time(r.station.at)} · main pollutant {r.station.pollutant}{r.offline ? ' · offline, cached' : ''}</div>
  {:else}
    <div class="dim small">{r.place?.name ?? ''}{r.offline ? ' · offline, cached' : ''}</div>
  {/if}
  {#if r.nearby.length}
    <h4 class="dim">Nearby stations</h4>
    {#each r.nearby as [s, d] (s.id)}
      <div class="near">
        <span class="dot" style:background={BANDS[bandOf(s.api ?? 0)].color}></span>
        <span class="ellipsis grow">{s.name}</span><span class="dim">{km(d)}</span><b>{Math.round(s.api ?? 0)}</b>
      </div>
    {/each}
  {/if}
  {#if r.place?.source === 'ip'}<p class="src dim">Approximate location from your IP.</p>{/if}
  <p class="src dim">Data: {r.kind === 'ipu' ? 'JAS/DOE APIMS, updated hourly' : 'Open-Meteo (US AQI scale, not IPU)'}</p>
{:else if ipu.loading}
  <p class="dim empty">Loading air quality…</p>
{:else}
  <p class="dim empty">No reading available. Check your internet connection or set a location.</p>
{/if}

<style>
  .hero { display: flex; flex-direction: column; align-items: center; padding: 18px; border-radius: 20px; margin-bottom: 12px;
          background: color-mix(in srgb, var(--c) 30%, var(--pill)); border: 1px solid color-mix(in srgb, var(--c) 60%, transparent); }
  .lbl { font-size: 12px; font-weight: 500; opacity: 0.8; }
  .big { font-size: 52px; font-weight: 700; line-height: 1.05; font-variant-numeric: tabular-nums; }
  .band { font-weight: 600; }
  .row { display: flex; align-items: center; gap: 6px; }
  .small { font-size: 12px; margin-top: 2px; }
  h4 { margin: 14px 2px 4px; font-size: 12px; font-weight: 500; }
  .near { display: flex; align-items: center; gap: 8px; padding: 6px 2px; border-bottom: 1px solid var(--line-soft); font-size: 12px; }
  .grow { flex: 1; }
  .dot { width: 9px; height: 9px; border-radius: 50%; flex: none; }
  .src { font-size: 11px; margin-top: 10px; }
  .empty { text-align: center; padding: 60px 10px; }
</style>
