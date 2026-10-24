<script lang="ts">
  // The pill is the progress bar (like Pomodoro): a fill behind the time, from the real end
  // time. Accent while running, dimmed and slowly pulsing when paused, orange in the final
  // minute, green with a flash when done (until dismissed). Several timers: the soonest,
  // plus "+N". The stopwatch has no end: no fill, an optional shimmer while it runs.
  import { AlarmClock, Pause, Play, Watch, X } from '@lucide/svelte';
  import { ipc, type Stopwatch, type Timer } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { tickingNow } from '../../lib/now.svelte';
  import { clock } from '../../lib/time';
  import type { PillProps } from '../types';
  import type { TimerSettings } from './index';
  import { fillFraction, pickTimer, stateColor, timerProgress } from './progress';

  let { size, settings }: PillProps<TimerSettings> = $props();
  const timers = live<Timer[]>('timers', ipc.timersList, []);
  const sw = live<Stopwatch | null>('stopwatch', ipc.swGet, null);
  // Ticks once a second only while something runs; CSS animates the fill in between.
  const t = tickingNow(() => timers.v.some((x) => x.endsAt !== null) || !!sw.v?.startedAt);
  const pick = $derived(pickTimer(timers.v, t.now));
  const p = $derived(pick.timer ? timerProgress(pick.timer, t.now) : null);
  const state = $derived(p && p.state === 'warn' && !settings.warn ? 'running' : p?.state);
  const act = (f: () => unknown) => (e: Event) => { e.stopPropagation(); f(); };
  const down = (e: PointerEvent) => e.stopPropagation();
</script>

<div class="pill tm {state ?? ''}" class:sw={!pick.timer && !!sw.v?.startedAt} class:shimmer={settings.shimmer}
     style:--c={state ? stateColor(state) : undefined}
     title={pick.timer ? `${pick.timer.label || 'Timer'} · ${state === 'done' ? 'finished' : state === 'paused' ? 'paused' : clock(p!.remainingMs) + ' left'}` : 'Timer & Stopwatch'}>
  {#if pick.timer && p && settings.progress}
    <span class="fill" style:transform="scaleX({state === 'done' ? 1 : fillFraction(p, settings.drain)})"></span>
  {/if}
  {#if pick.timer && p}
    <AlarmClock size={15} />
    <span class="t">{state === 'done' ? 'Done' : clock(p.remainingMs)}</span>
    {#if size !== 'small' && pick.timer.label}<span class="dim ellipsis lbl">{pick.timer.label}</span>{/if}
    {#if pick.others}<span class="more" title="{pick.others} more running">+{pick.others}</span>{/if}
    {#if state === 'done'}
      <button class="ctl" onpointerdown={down} onclick={act(() => ipc.timerDelete(pick.timer!.id))} aria-label="Dismiss" title="Dismiss"><X size={13} /></button>
    {:else if state === 'paused'}
      <button class="ctl" onpointerdown={down} onclick={act(() => ipc.timerResume(pick.timer!.id))} aria-label="Resume" title="Resume"><Play size={13} /></button>
    {:else}
      <button class="ctl" onpointerdown={down} onclick={act(() => ipc.timerPause(pick.timer!.id))} aria-label="Pause" title="Pause"><Pause size={13} /></button>
    {/if}
  {:else if sw.v?.startedAt}
    <Watch size={15} /><span class="t">{clock(sw.v.accMs + t.now - sw.v.startedAt)}</span>
  {:else}
    <AlarmClock size={16} />
  {/if}
</div>

<style>
  .tm { position: relative; overflow: hidden; gap: 6px; }
  .tm > :not(.fill) { position: relative; z-index: 1; }
  /* The fill: a rectangle behind the text, animated by CSS between the 1 s updates. */
  .fill {
    position: absolute; inset: 0; z-index: 0; transform-origin: left center;
    background: color-mix(in srgb, var(--c) 38%, transparent);
    border-right: 2px solid var(--c);
    transition: transform 1s linear, background 0.4s;
  }
  .paused .fill { opacity: 0.55; animation: breathe 2.6s ease-in-out infinite; }
  @keyframes breathe { 50% { opacity: 0.3; } }
  .done .fill { border-right: 0; background: color-mix(in srgb, var(--c) 55%, transparent); animation: flash 0.5s ease-in-out 4; }
  @keyframes flash { 50% { background: color-mix(in srgb, var(--c) 85%, transparent); } }
  /* Readable over any fill. */
  .t { font-variant-numeric: tabular-nums; font-weight: 600; text-shadow: 0 0 3px var(--pill); }
  .lbl { max-width: 90px; font-size: 12px; }
  .more { font-size: 10px; font-weight: 600; padding: 1px 5px; border-radius: calc(8px * var(--rs)); background: var(--pill-hover); }
  .ctl { display: grid; place-items: center; width: 22px; height: 22px; border-radius: 50%; margin-right: -4px; transition: background 0.2s, transform 0.25s var(--spring); }
  .ctl:hover { background: var(--pill-hover); }
  .ctl:active { transform: scale(0.85); }
  /* Stopwatch running: a soft light sweeping across (optional). */
  .sw.shimmer::after {
    content: ''; position: absolute; inset: 0; z-index: 0; pointer-events: none;
    background: linear-gradient(100deg, transparent 30%, color-mix(in srgb, var(--accent) 22%, transparent) 50%, transparent 70%);
    background-size: 250% 100%; animation: sweep 2.8s linear infinite;
  }
  @keyframes sweep { from { background-position: 130% 0; } to { background-position: -130% 0; } }
</style>
