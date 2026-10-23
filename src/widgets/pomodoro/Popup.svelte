<script lang="ts">
  // Pomodoro panel (design rules: header · hero · sections · footer).
  // Hero: the ring with the time left; the ring and status text take the phase colour
  // (focus = red, break = green/blue) because the colour means something. One primary
  // button (start / pause / resume); reset and skip are ghost icons beside it.
  import { Minus, Pause, Play, Plus, RotateCcw, Settings2, SkipForward } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc, type PomoCfg } from '../../lib/ipc';
  import PanelHeader from '../../lib/PanelHeader.svelte';
  import Ring from '../../lib/Ring.svelte';
  import { clock, fmtMinutes } from '../../lib/time';
  import type { PopupProps } from '../types';
  import type { ProductivitySettings } from '../productivity/index';
  import { PHASE, stateOf } from './index';
  import { livePomo } from './pomo.svelte';

  let { instance }: PopupProps<ProductivitySettings> = $props();
  const p = livePomo();
  const phase = $derived(p.state ? PHASE[p.state.pomo.phase] : PHASE.work);
  const st = $derived(p.state ? stateOf(p.state.pomo) : null);
  const main = () => (p.running ? ipc.pomoPause() : ipc.pomoStart());
  const every = $derived(Math.max(1, p.state?.cfg.every ?? 4));
  const done = $derived(Math.min(p.state?.pomo.done ?? 0, every));
  const onWork = $derived(p.state?.pomo.phase === 'work');
  let showCfg = $state(false);

  const FIELDS: [keyof PomoCfg, string, number][] = [
    ['workMin', 'Focus', 180], ['shortMin', 'Short break', 60], ['longMin', 'Long break', 120], ['every', 'Sessions per round', 12],
  ];
  // Minutes move by 5 (by 1 at 5 and below); the session count by 1.
  const stepOf = (k: keyof PomoCfg, v: number, dir: 1 | -1) => (k === 'every' || (dir < 0 ? v <= 5 : v < 5) ? 1 : 5) * dir;
  const setCfg = (k: keyof PomoCfg, max: number, v: number) => updateWidgetSettings(instance, { [k]: Math.min(max, Math.max(1, Math.round(v))) });
</script>

<PanelHeader title="Pomodoro">
  <button class="icon-btn" class:on={showCfg} onclick={() => (showCfg = !showCfg)} aria-label="Durations" title="Durations"><Settings2 size={16} /></button>
</PanelHeader>

<section class="hero" style:--c={phase.color}>
  <span class="status">{st?.title ?? ''}</span>
  <Ring value={st?.key === 'completed' ? 1 : p.progress} size={184} stroke={8} color={phase.color}>
    <span class="time" class:paused={st?.key === 'paused'}>{st?.key === 'completed' ? '✓' : clock(p.remaining)}</span>
    <span class="phase">{phase.name}</span>
  </Ring>
  <div class="dots" role="img" aria-label="Session {Math.min(done + (onWork ? 1 : 0), every)} of {every}">
    {#each Array(every) as _, i (i)}
      <i class:full={i < done} class:now={onWork && i === done}></i>
    {/each}
  </div>
  <div class="controls">
    <button class="ghost" onclick={() => ipc.pomoReset()} title="Reset to the first focus session" aria-label="Reset"><RotateCcw size={16} /></button>
    <button class="primary" onclick={main}>
      {#if p.running}<Pause size={16} />{:else}<Play size={16} />{/if}
      {st?.action ?? 'Start'}
    </button>
    <button class="ghost" disabled={st?.key === 'completed'} onclick={() => ipc.pomoSkip()} title="Skip to the next phase (it waits for your click)" aria-label="Skip phase"><SkipForward size={16} /></button>
  </div>
</section>

<hr />
<section class="today">
  <span>Focused today</span>
  <b>{fmtMinutes(p.state?.today.minutes ?? 0)}<span class="sub">&nbsp;·&nbsp;{p.state?.today.sessions ?? 0} {p.state?.today.sessions === 1 ? 'session' : 'sessions'}</span></b>
</section>

{#if showCfg && p.state}
  <hr />
  <section class="cfg" aria-label="Durations">
    {#each FIELDS as [k, label, max] (k)}
      {@const v = p.state.cfg[k]}
      <div class="row">
        <span>{label}</span>
        <span class="step">
          <button class="ghost sm" disabled={v <= 1} onclick={() => setCfg(k, max, v + stepOf(k, v, -1))} aria-label="Less {label}"><Minus size={14} /></button>
          <b>{v}{k === 'every' ? '' : ' min'}</b>
          <button class="ghost sm" disabled={v >= max} onclick={() => setCfg(k, max, v + stepOf(k, v, 1))} aria-label="More {label}"><Plus size={14} /></button>
        </span>
      </div>
    {/each}
  </section>
{/if}

<p class="foot">Phases start only when you click.</p>

<style>
  .icon-btn.on { color: var(--accent); }
  .hero { display: flex; flex-direction: column; align-items: center; gap: var(--s-3); padding-bottom: var(--s-2); }
  .status { font-size: 14px; font-weight: 600; color: var(--c); }
  .time { font-size: 48px; font-weight: 600; line-height: 1; letter-spacing: -0.02em; }
  .time.paused { color: var(--fg-dim); }
  .phase { font-size: 12px; color: var(--fg-dim); margin-top: var(--s-1); }
  /* Session dots: done = filled, the current focus session = ring, rest = track. */
  .dots { display: flex; gap: var(--s-2); }
  .dots i { width: 8px; height: 8px; border-radius: 50%; background: var(--track); }
  .dots i.full { background: var(--c); }
  .dots i.now { background: transparent; box-shadow: inset 0 0 0 2px var(--c); }
  .controls { display: flex; align-items: center; gap: var(--s-3); }
  .primary {
    display: inline-flex; align-items: center; gap: var(--s-2); height: 36px; padding: 0 var(--s-4);
    border-radius: var(--r-chip); background: var(--accent); color: #fff; font-weight: 600;
    transition: filter 0.15s, transform 0.2s var(--spring);
  }
  .primary:hover { filter: brightness(1.1); }
  .primary:active { transform: scale(0.96); }
  .primary:focus-visible, .ghost:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .ghost { display: grid; place-items: center; width: 32px; height: 32px; border-radius: var(--r-inner); color: var(--fg-dim); transition: background 0.15s, color 0.15s; }
  .ghost:hover:not(:disabled) { background: var(--pill-hover); color: var(--fg); }
  .ghost:active:not(:disabled) { transform: scale(0.92); }
  .ghost:disabled { opacity: 0.35; }
  .ghost.sm { width: 28px; height: 28px; }
  hr { border: 0; height: 1px; background: var(--border); margin: var(--s-4) 0; }
  .today, .row { display: flex; align-items: center; justify-content: space-between; min-height: 40px; padding: 0 var(--s-1); }
  .today span, .row > span:first-child { color: var(--fg-dim); }
  .today b { font-size: 14px; font-weight: 600; }
  .today .sub { font-weight: 400; color: var(--fg-dim); }
  .step { display: flex; align-items: center; gap: var(--s-1); }
  .step b { min-width: 56px; text-align: center; font-weight: 600; }
  .foot { margin: var(--s-3) 0 0; font-size: 11px; color: var(--fg-3); text-align: center; }
  @media (prefers-reduced-motion: reduce) { .primary, .ghost { transition: none; } }
</style>
