<script lang="ts">
  import { ipc } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import type { ClockSettings } from './index';
  import { fmtTime, ticker } from './time.svelte';
  import { zonesOf } from './zones';

  let { instance, size, settings, vertical }: PillProps<ClockSettings> = $props();
  const clock = ticker(() => settings.showSeconds);
  const date = $derived(new Intl.DateTimeFormat(undefined, { weekday: 'short', day: 'numeric', month: 'short' }).format(clock.now));
  const first = $derived(zonesOf(settings)[0]);

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

<div class="pill clock {settings.appearance}" class:vertical onpointerenter={enter} onpointerleave={leave} role="timer">
  {#if settings.appearance === 'prominent' && !vertical}
    <span class="big">{fmtTime(clock.now, settings.hour12, settings.showSeconds)}</span>
    {#if size !== 'small'}<span class="date">{date}</span>{/if}
  {:else}
    {#if size !== 'small'}<span class="dim">{date}</span>{/if}
    <span class="time">{fmtTime(clock.now, settings.hour12, settings.showSeconds)}</span>
  {/if}
  {#if size === 'large' && first && !vertical}
    <span class="dim zone">{first.label} {fmtTime(clock.now, settings.hour12, false, first.tz)}</span>
  {/if}
</div>

<style>
  .time { font-weight: 600; font-variant-numeric: tabular-nums; }
  .vertical { padding: 0 4px; font-size: 11px; }
  .zone { font-size: 12px; }
  /* prominent: larger, bolder time on an accent-tinted panel, still within the bar height */
  .prominent {
    background: color-mix(in srgb, var(--accent) 30%, var(--pill));
    border-color: color-mix(in srgb, var(--accent) 55%, transparent);
    gap: 10px; padding: 0 14px;
  }
  .prominent:hover { background: color-mix(in srgb, var(--accent) 42%, var(--pill)); }
  .big { font-size: 19px; font-weight: 700; letter-spacing: 0.3px; font-variant-numeric: tabular-nums; line-height: 1; }
  .date { font-size: 12px; font-weight: 500; opacity: 0.85; }
</style>
