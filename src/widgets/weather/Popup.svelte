<script lang="ts">
  import { Droplets, MapPin, RefreshCw, Thermometer, Umbrella, Wind } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import { describe } from './codes';
  import { compass, themeOf } from './sun';
  import { deg, liveWeather } from './weather.svelte';
  import SunArc from './SunArc.svelte';
  import WxIcon from './WxIcon.svelte';

  // The forecast panel, opened from the weather part of the clock (it has no settings of its own).
  const wx = liveWeather();
  let refreshing = $state(false);
  let now = $state(Date.now());
  $effect(() => { const t = setInterval(() => (now = Date.now()), 60_000); return () => clearInterval(t); });
  const hour = (ms: number) => new Intl.DateTimeFormat(undefined, { hour: 'numeric' }).format(ms);
  const day = (ms: number, i: number) => (i === 0 ? 'Today' : new Intl.DateTimeFormat(undefined, { weekday: 'long' }).format(ms));
  const time = (ms: number) => new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' }).format(ms);
  const pct = (v?: number | null) => (v == null ? '–' : `${Math.round(v)}%`);

  async function refresh() {
    refreshing = true;
    wx.w = (await ipc.weatherGet(true)) ?? wx.w;
    refreshing = false;
  }
</script>

{#if wx.w}
  {@const w = wx.w}
  {@const d0 = w.daily[0]}
  {@const desc = describe(w.current.code, w.current.isDay)}
  <div class="hero" style:background={themeOf(desc.icon, w.current.isDay)}>
    <div class="top">
      <div>
        <div class="place"><MapPin size={13} /> {w.place?.name || 'Unknown place'}</div>
        <div class="sub">
          {w.offline ? 'Offline · last updated' : 'Updated'} {time(w.fetchedAt)}
          {#if w.place?.source === 'ip'} · approximate (IP){/if}
        </div>
      </div>
      <button class="icon-btn" class:spin={refreshing} onclick={refresh} aria-label="Refresh"><RefreshCw size={15} /></button>
    </div>
    <div class="now">
      <WxIcon code={w.current.code} isDay={w.current.isDay} size={46} />
      <span class="big">{deg(w.current.temp)}</span>
      <div class="lbl">
        <b>{desc.label}</b>
        {#if d0}<span>H {deg(d0.max)} · L {deg(d0.min)}</span>{/if}
      </div>
    </div>
  </div>

  <div class="stats">
    <div class="card s"><Thermometer size={14} /><span class="dim">Feels like</span><b>{w.current.feelsLike == null ? '–' : deg(w.current.feelsLike)}</b></div>
    <div class="card s"><Droplets size={14} /><span class="dim">Humidity</span><b>{pct(w.current.humidity)}</b></div>
    <div class="card s"><Wind size={14} /><span class="dim">Wind</span>
      <b>{w.current.windSpeed == null ? '–' : `${Math.round(w.current.windSpeed)} ${w.unit === 'f' ? 'mph' : 'km/h'}`}{#if w.current.windDir != null}&nbsp;{compass(w.current.windDir)}{/if}</b></div>
    <div class="card s"><Umbrella size={14} /><span class="dim">Rain chance</span><b>{pct(d0?.pop)}</b></div>
  </div>

  <div class="card sun"><SunArc rise={d0?.sunrise} set={d0?.sunset} {now} /></div>

  <div class="hours card">
    {#each w.hourly as h (h.time)}
      <div class="h"><span class="dim small">{hour(h.time)}</span><WxIcon code={h.code} size={18} /><span>{deg(h.temp)}</span>
        {#if h.pop != null}<span class="pop">{Math.round(h.pop)}%</span>{/if}</div>
    {/each}
  </div>
  <div class="days">
    {#each w.daily as d, i (d.date)}
      <div class="d"><span class="dn">{day(d.date, i)}</span><span class="pop">{d.pop != null ? `${Math.round(d.pop)}%` : ''}</span><WxIcon code={d.code} size={18} /><span class="dim">{deg(d.min)}</span><span>{deg(d.max)}</span></div>
    {/each}
  </div>
  <p class="dim small src">Open-Meteo forecast</p>
{:else if wx.loading}
  <p class="dim empty">Loading weather…</p>
{:else}
  <p class="dim empty">No weather yet. Check your internet connection, or set a city in Settings → Widgets → Clock & Date.</p>
{/if}

<style>
  .hero { border-radius: 16px; padding: 12px 14px; color: #fff; text-shadow: 0 1px 2px rgba(0,0,0,0.2); }
  .top { display: flex; justify-content: space-between; align-items: flex-start; }
  .place { display: flex; align-items: center; gap: 4px; font-weight: 600; }
  .sub { font-size: 11px; opacity: 0.85; }
  .hero .icon-btn { color: #fff; }
  .now { display: flex; align-items: center; gap: 12px; margin-top: 10px; }
  .big { font-size: 44px; font-weight: 600; line-height: 1; }
  .lbl { display: flex; flex-direction: column; font-size: 12px; gap: 2px; }
  .stats { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; margin: 8px 0; }
  .s { display: grid; grid-template-columns: 16px 1fr; grid-template-rows: auto auto; column-gap: 6px; padding: 8px 10px; font-size: 11px; align-items: center; }
  .s b { grid-column: 2; font-size: 14px; }
  .sun { padding: 10px 12px 8px; margin-bottom: 8px; }
  .small { font-size: 11px; }
  .hours { display: flex; gap: 14px; overflow-x: auto; padding: 10px 12px; scrollbar-width: thin; }
  .h { display: flex; flex-direction: column; align-items: center; gap: 3px; flex: none; }
  .pop { font-size: 10px; color: #64d2ff; }
  .days { margin-top: 8px; }
  .d { display: grid; grid-template-columns: 1fr 32px 24px 40px 40px; align-items: center; gap: 8px; padding: 7px 4px; border-bottom: 1px solid var(--line-soft); text-align: right; }
  .dn { text-align: left; }
  .src { margin: 8px 0 0; text-align: right; }
  .spin :global(svg) { animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .empty { text-align: center; padding: 60px 10px; }
</style>
