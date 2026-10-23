<script lang="ts">
  import { MapPin, RefreshCw } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import type { PopupProps } from '../types';
  import { describe } from './codes';
  import { deg, liveWeather } from './weather.svelte';
  import WxIcon from './WxIcon.svelte';

  let {}: PopupProps = $props();
  const wx = liveWeather();
  let refreshing = $state(false);
  const hour = (ms: number) => new Intl.DateTimeFormat(undefined, { hour: 'numeric' }).format(ms);
  const day = (ms: number, i: number) => (i === 0 ? 'Today' : new Intl.DateTimeFormat(undefined, { weekday: 'long' }).format(ms));
  const time = (ms: number) => new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' }).format(ms);

  async function refresh() {
    refreshing = true;
    wx.w = (await ipc.weatherGet(true)) ?? wx.w;
    refreshing = false;
  }
</script>

{#if wx.w}
  {@const w = wx.w}
  <div class="top">
    <div>
      <div class="place"><MapPin size={13} /> {w.place?.name || 'Unknown place'}</div>
      <div class="dim small">
        {w.offline ? 'Offline · showing' : 'Updated'} {time(w.fetchedAt)}
        {#if w.place?.source === 'ip'} · Approximate location from your IP{/if}
      </div>
    </div>
    <button class="icon-btn" class:spin={refreshing} onclick={refresh} aria-label="Refresh"><RefreshCw size={15} /></button>
  </div>
  <div class="now">
    <WxIcon code={w.current.code} isDay={w.current.isDay} size={46} />
    <span class="big">{deg(w.current.temp)}</span>
    <span class="dim">{describe(w.current.code, w.current.isDay).label}</span>
  </div>
  <div class="hours card">
    {#each w.hourly as h (h.time)}
      <div class="h"><span class="dim small">{hour(h.time)}</span><WxIcon code={h.code} size={18} /><span>{deg(h.temp)}</span></div>
    {/each}
  </div>
  <div class="days">
    {#each w.daily as d, i (d.date)}
      <div class="d"><span class="dn">{day(d.date, i)}</span><WxIcon code={d.code} size={18} /><span class="dim">{deg(d.min)}</span><span>{deg(d.max)}</span></div>
    {/each}
  </div>
{:else if wx.loading}
  <p class="dim empty">Loading weather…</p>
{:else}
  <p class="dim empty">No weather yet. Check your internet connection, or set a city in Settings → Widgets → Weather.</p>
{/if}

<style>
  .top { display: flex; justify-content: space-between; align-items: flex-start; }
  .place { display: flex; align-items: center; gap: 4px; font-weight: 500; }
  .small { font-size: 11px; }
  .now { display: flex; align-items: center; gap: 12px; margin: 14px 0; }
  .big { font-size: 44px; font-weight: 600; line-height: 1; }
  .hours { display: flex; gap: 14px; overflow-x: auto; padding: 10px 12px; scrollbar-width: thin; }
  .h { display: flex; flex-direction: column; align-items: center; gap: 4px; flex: none; }
  .days { margin-top: 10px; }
  .d { display: grid; grid-template-columns: 1fr 24px 40px 40px; align-items: center; gap: 8px; padding: 7px 4px; border-bottom: 1px solid var(--line-soft); text-align: right; }
  .dn { text-align: left; }
  .spin :global(svg) { animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .empty { text-align: center; padding: 60px 10px; }
</style>
