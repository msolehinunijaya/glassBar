<script lang="ts">
  import { ipc } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import type { ClockSettings } from './index';
  import { date } from './prefs.svelte';
  import Time from './ClockTime.svelte';
  import { ticker } from './time.svelte';
  import { zonesOf } from './zones';

  let { instance, size, settings, vertical }: PillProps<ClockSettings> = $props();
  // Ticks every second only while seconds are shown; otherwise on each minute boundary.
  const clock = ticker(() => settings.showSeconds);
  const first = $derived(zonesOf(settings)[0]);
  // The small size (and a side dock) always shows the time only.
  const show = $derived(size === 'small' || vertical ? 'time' : settings.show);
  const day = $derived(date(clock.now, settings));

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

<div class="pill clock {settings.appearance}" class:vertical class:stacked={show === 'both' && settings.layout === 'stacked'}
     onpointerenter={enter} onpointerleave={leave} role="timer" title={day}>
  {#if show !== 'date'}
    <span class="time" class:big={settings.appearance === 'prominent' && !vertical}><Time now={clock.now} {settings} /></span>
  {/if}
  {#if show === 'both' && settings.layout === 'inline'}<span class="dot dim">·</span>{/if}
  {#if show !== 'time'}<span class="date" class:dim={show === 'both'}>{day}</span>{/if}
  {#if size === 'large' && first && !vertical}
    <span class="dim zone">{first.label} <Time now={clock.now} {settings} seconds={settings.worldSeconds} tz={first.tz} /></span>
  {/if}
</div>

<style>
  .time { font-weight: 600; }
  .date { white-space: nowrap; font-variant-numeric: tabular-nums; }
  .dot { margin: 0 -2px; }
  .vertical { padding: 0 4px; font-size: 11px; }
  .zone { font-size: 12px; }
  /* stacked: time above the date */
  .stacked { flex-direction: column; justify-content: center; gap: 0; line-height: 1.15; }
  .stacked .date { font-size: 11px; }
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
