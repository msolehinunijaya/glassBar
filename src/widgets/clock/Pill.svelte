<script lang="ts">
  import { ipc } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import type { ClockSettings } from './index';
  import { date, time } from './prefs.svelte';
  import Time from './ClockTime.svelte';
  import { ticker } from './time.svelte';
  import { zonesOf } from './zones';
  import WeatherPart from './WeatherPart.svelte';

  let { instance, size, settings, vertical }: PillProps<ClockSettings> = $props();
  // Ticks every second only while seconds are shown; otherwise on each minute boundary.
  const clock = ticker(() => settings.showSeconds);
  const first = $derived(zonesOf(settings)[0]);
  // The small size always shows the time only.
  const show = $derived(size === 'small' ? 'time' : settings.show);
  const day = $derived(date(clock.now, settings));
  // Left/right dock: time without seconds, AM/PM under it, then a short weekday + day.
  const sideTime = $derived(time(clock.now, settings, false));
  const sideDay = $derived(new Intl.DateTimeFormat(undefined, { weekday: 'short', day: 'numeric' }).format(clock.now));

  // "Popup" appearance: hovering the clock shows the big clock + world clocks, without a click.
  let hoverTimer: ReturnType<typeof setTimeout>;
  function enter(e: PointerEvent) {
    if (settings.appearance !== 'popup') return;
    const el = e.currentTarget as HTMLElement;
    hoverTimer = setTimeout(() => ipc.popupOpen(`__hover:${instance}`, el.getBoundingClientRect(), 340, 560, false), 400);
  }
  function leave() {
    clearTimeout(hoverTimer);
    if (settings.appearance === 'popup') ipc.popupLeave();
  }
</script>

{#if vertical}
<div class="side-stack" onpointerenter={enter} onpointerleave={leave} role="timer" title={day}>
  {#if settings.showWeather}<WeatherPart {instance} colorful={settings.weatherColorful} vertical />{/if}
  <div class="side">
    <span class="side-v">{sideTime.main}</span>
    {#if sideTime.ampm && settings.ampmSize !== 'hidden'}<span class="side-k">{sideTime.ampm}</span>{/if}
    <span class="side-c">{sideDay}</span>
  </div>
</div>
{:else}
<div class="clockwrap" class:withwx={settings.showWeather}>
{#if settings.showWeather}<WeatherPart {instance} colorful={settings.weatherColorful} vertical={false} />{/if}
<div class="pill clock {settings.appearance}" class:stacked={show === 'both' && settings.layout === 'stacked'}
     class:date-first={show === 'both' && settings.timeSide === 'right'}
     data-align={settings.stackAlign}
     onpointerenter={enter} onpointerleave={leave} role="timer" title={day}>
  {#if show !== 'date'}
    <span class="time" class:big={settings.appearance === 'prominent'}><Time now={clock.now} {settings} /></span>
  {/if}
  {#if show === 'both' && settings.layout === 'inline'}<span class="dot dim">·</span>{/if}
  {#if show !== 'time'}<span class="date" class:dim={show === 'both'}>{day}</span>{/if}
  {#if size === 'large' && first}
    <span class="dim zone">{first.label} <Time now={clock.now} {settings} seconds={settings.worldSeconds} tz={first.tz} /></span>
  {/if}
</div>
</div>
{/if}

<style>
  /* Weather (optional) sits inside the same pill shape, on the left. */
  .clockwrap { display: flex; align-items: stretch; gap: 2px; height: 100%; }
  .withwx:not(.vertical) > .clock { border-top-left-radius: 4px; border-bottom-left-radius: 4px; }
  .time { font-weight: 600; }
  .date { white-space: nowrap; font-variant-numeric: tabular-nums; }
  .dot { margin: 0 -2px; }
  /* date first: reverse the order (left/right, or top/bottom when stacked) */
  .date-first .time { order: 3; }
  .date-first .dot { order: 2; }
  .date-first .date { order: 1; }
  .date-first .zone { order: 4; }
  .zone { font-size: 12px; }
  /* stacked: time above the date */
  .stacked { flex-direction: column; justify-content: center; gap: 0; line-height: 1.15; }
  .stacked .date { font-size: 11px; }
  .stacked[data-align='left'] { align-items: flex-start; text-align: left; }
  .stacked[data-align='center'] { align-items: center; text-align: center; }
  .stacked[data-align='right'] { align-items: flex-end; text-align: right; }
  /* prominent: larger, bolder time on an accent-tinted panel, still within the bar height */
  .prominent {
    background: color-mix(in srgb, var(--accent) 30%, var(--pill));
    border-color: color-mix(in srgb, var(--accent) 55%, transparent);
    gap: 10px; padding: 0 14px;
  }
  .prominent:hover { background: color-mix(in srgb, var(--accent) 42%, var(--pill)); }
  .big { font-size: 19px; font-weight: 700; letter-spacing: 0.3px; line-height: 1; }
  .prominent .date { font-size: 12px; font-weight: 500; opacity: 0.85; }
  .prominent.stacked { gap: 2px; }
  .prominent.stacked .big { font-size: 16px; }
</style>
