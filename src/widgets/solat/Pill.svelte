<script lang="ts">
  // "Asar in 25m" over a progress fill (previous prayer → next). Last 15 min: amber with a
  // gentle pulse. When the time comes: a flashing glow and a ✓ button; clicking ✓ stops
  // the flashing and moves on to the next prayer. (The alert/azan sound is the existing
  // prayer alert, switchable in the widget settings.)
  import { Check, Moon, Square } from '@lucide/svelte';
  import { adhan, stopAdhan } from '../../lib/adhan.svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { stopSound } from '../../lib/sound';
  import type { PillProps } from '../types';
  import type { SolatSettings } from './index';
  import { countdown } from './next';
  import { liveSolat } from './solat.svelte';
  import { prayerState } from './state';

  let { instance, size, settings }: PillProps<SolatSettings> = $props();
  const s = liveSolat(() => settings);
  const st = $derived(s.info ? prayerState(s.info.days, s.now, settings.ackKey || null, settings.showImsak) : null);
  const at = (ms: number) => new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' }).format(ms);

  function ack(e: Event) {
    e.stopPropagation();
    stopSound();
    if (st?.reached) updateWidgetSettings(instance, { ackKey: st.reached.key });
  }
</script>

<div class="pill solat {st?.phase ?? ''}" title={s.error || (st ? `${st.next.name} at ${at(st.next.at)}` : 'Waktu Solat')}>
  {#if st && st.phase !== 'reached'}<span class="fill" style:transform="scaleX({st.progress})"></span>{/if}
  <Moon size={15} />
  {#if st?.phase === 'reached' && st.reached}
    <span class="stack"><span class="name">{st.reached.name}</span><span class="now">time now</span></span>
    {#if !adhan.playing}<button class="ok" onclick={ack} onpointerdown={(e) => e.stopPropagation()} aria-label="Acknowledge {st.reached.name}"><Check size={14} strokeWidth={3} /></button>{/if}
  {:else if st}
    <!-- Stacked: name on top, countdown below (compact on the dock). -->
    <span class="stack">
      <span class="name">{st.next.name}</span>
      {#if size !== 'small'}<span class="t">in {countdown(st.next.at - s.now)}{#if size === 'large'}<span class="dim">&nbsp;·&nbsp;{at(st.next.at)}</span>{/if}</span>{/if}
    </span>
  {:else}
    <span class="dim">–</span>
  {/if}
  {#if adhan.playing}
    <!-- The adzan is playing: stop it right here. -->
    <button class="stop" onclick={(e) => { e.stopPropagation(); stopAdhan(); }} onpointerdown={(e) => e.stopPropagation()}
            aria-label="Stop the adhan" title="Stop the adhan"><Square size={10} fill="currentColor" /></button>
  {/if}
</div>

<style>
  .solat { position: relative; overflow: hidden; }
  .solat > :not(.fill) { position: relative; z-index: 1; }
  .fill {
    position: absolute; inset: 0; z-index: 0; transform-origin: left center;
    background: color-mix(in srgb, var(--accent) 24%, transparent);
    transition: transform 1s linear, background 0.4s;
  }
  .solat { gap: 6px; padding: 0 10px 0 8px; }
  .stop {
    display: grid; place-items: center; width: 24px; height: 24px; flex: none; border-radius: 50%;
    background: #ff453a; color: #fff; animation: pulse 1.6s ease-in-out infinite;
    transition: transform 0.2s var(--spring, ease);
  }
  .stop:hover { transform: scale(1.1); }
  .stop:active { transform: scale(0.9); }
  @keyframes pulse { 50% { box-shadow: 0 0 0 4px rgba(255, 69, 58, 0.25); } }
  @media (prefers-reduced-motion: reduce) { .stop { animation: none; } }
  .stack { display: flex; flex-direction: column; line-height: 1.15; }
  .name { font-weight: 600; font-size: 12px; }
  .t { font-variant-numeric: tabular-nums; font-size: 10.5px; color: var(--fg-dim); white-space: nowrap; }
  .approaching .fill { background: color-mix(in srgb, #ffb340 40%, transparent); }
  .approaching { animation: soft 2.4s ease-in-out infinite; }
  @keyframes soft { 50% { box-shadow: inset 0 0 0 1px #ffb340, 0 0 10px rgba(255, 179, 64, 0.35); } }
  .reached { background: color-mix(in srgb, #30d158 30%, var(--pill)); animation: glow 1.1s ease-in-out infinite; }
  @keyframes glow { 50% { box-shadow: 0 0 0 2px #30d158, 0 0 16px rgba(48, 209, 88, 0.7); background: color-mix(in srgb, #30d158 55%, var(--pill)); } }
  .now { font-size: 10.5px; }
  .ok { display: grid; place-items: center; width: 24px; height: 24px; border-radius: 50%; background: #30d158; color: #fff; transition: transform 0.3s var(--spring); }
  .ok:active { transform: scale(0.85); }
</style>
