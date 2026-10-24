<script lang="ts">
  // Pomodoro flyout. One phase colour (--c) drives the card: ring, dots, primary button,
  // goal bar. Header · Focus/Short/Long · ring · session dots · controls · Today.
  // Keys: Space start/pause · R reset · S skip · 1/2/3 phase.
  import { Coffee, Pause, Play, RotateCcw, Settings2, SkipForward } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc, type Phase } from '../../lib/ipc';
  import PanelHeader from '../../lib/PanelHeader.svelte';
  import { clock, fmtMinutes } from '../../lib/time';
  import type { PopupProps } from '../types';
  import type { ProductivitySettings } from '../productivity/index';
  import { PHASE, PHASES } from './index';
  import PhaseRing from './PhaseRing.svelte';
  import { livePomo, toggle } from './pomo.svelte';
  import PomoSettings from './SettingsForm.svelte';

  let { instance, settings }: PopupProps<ProductivitySettings> = $props();
  const p = livePomo();
  const v = $derived(p.view);
  const color = $derived(PHASE[v?.phase ?? 'work'].color);
  const paused = $derived(v?.status === 'paused');
  const urgent = $derived(v?.status === 'running' && p.remaining <= 60_000);
  const time = $derived(clock(p.remaining)); // "24:59" or "1:59:59"; the last colon blinks
  const mm = $derived(time.slice(0, time.lastIndexOf(':')));
  const ss = $derived(time.slice(time.lastIndexOf(':') + 1));
  let showCfg = $state(false);
  let more = $state(false);
  /** An inline "are you sure" in place of the controls. */
  let ask = $state<{ text: string; yes: () => unknown } | null>(null);

  const main = () => toggle(p.running);
  function reset() {
    if (p.elapsed > 5 * 60_000) ask = { text: 'Reset?', yes: ipc.pomoReset };
    else ipc.pomoReset();
  }
  function setPhase(ph: Phase) {
    if (!v || ph === v.phase) return;
    if (v.status !== 'ready') ask = { text: 'End this session?', yes: () => ipc.pomoSetPhase(ph) };
    else ipc.pomoSetPhase(ph);
  }
  const answer = (yes: boolean) => { if (yes) ask?.yes(); ask = null; };
  const resetTitle = $derived(v?.status === 'ready' ? 'Reset the round' : 'Reset timer');

  function onKey(e: KeyboardEvent) {
    if (showCfg || e.ctrlKey || e.altKey || e.metaKey || !v) return;
    const t = e.target instanceof Element ? e.target : document.body;
    if (t.closest('input, textarea, select, [contenteditable]')) return;
    if (ask) { if (e.key === 'y' || e.key === 'Enter') answer(true); else if (e.key === 'n') answer(false); else return; }
    else if (e.key === ' ') { if (t.closest('button')) return; main(); }
    else if (e.key === 'r' || e.key === 'R') reset();
    else if (e.key === 's' || e.key === 'S') ipc.pomoSkip();
    else if (e.key >= '1' && e.key <= '3') setPhase(PHASES[+e.key - 1]);
    else return;
    e.preventDefault();
  }

  const save = (patch: Partial<ProductivitySettings>) => updateWidgetSettings(instance, patch);
  const week = $derived(p.state?.week ?? []);
  const peak = $derived(Math.max(1, ...week.map((d) => d.minutes)));
  const dayName = (d: string, style: 'narrow' | 'long') => { const [y, m, n] = d.split('-').map(Number); return new Date(y, m - 1, n).toLocaleDateString(undefined, { weekday: style }); };
</script>

<svelte:window onkeydown={onKey} />

<PanelHeader title="Pomodoro">
  <button class="icon-btn" class:on={showCfg} onclick={() => (showCfg = !showCfg)} aria-label="Pomodoro settings" aria-pressed={showCfg} title="Settings"><Settings2 size={16} /></button>
</PanelHeader>

{#if showCfg}
  <PomoSettings {instance} {settings} {save} />
{:else if v}
<div class="pomo" style:--c={color}>
  <div class="seg" role="radiogroup" aria-label="Phase">
    {#each PHASES as ph (ph)}
      <button role="radio" aria-checked={v.phase === ph} class:on={v.phase === ph} style:--c={PHASE[ph].color}
              onclick={() => setPhase(ph)} title="{PHASE[ph].name} ({PHASE[ph].key})">{ph === 'work' ? 'Focus' : ph === 'short' ? 'Short' : 'Long'}</button>
    {/each}
  </div>

  <div class="hero" aria-live="polite">
    <PhaseRing size={200} stroke={10} left={p.left} {paused} {urgent} ticking={p.running}>
      <span class="time" class:urgent class:long={time.length > 5} aria-label="{time} left">{mm}<span class="colon" class:blink={paused}>:</span>{ss}</span>
      <span class="state">{#if paused}<Pause size={12} strokeWidth={2.5} />{/if}{v.label}</span>
    </PhaseRing>
  </div>

  <div class="dots" aria-hidden="true">
    {#each v.dots as d, i (i)}<i class={d} class:live={d === 'current' && v.status === 'running'}></i>{/each}
  </div>
  <p class="session">{v.session}</p>

  <div class="controls">
    {#if ask}
      <span class="ask">{ask.text}</span>
      <button class="chip yes" onclick={() => answer(true)}>Yes</button>
      <button class="chip" onclick={() => answer(false)}>No</button>
    {:else}
      <button class="round" onclick={reset} title="{resetTitle} (R)" aria-label={resetTitle}><RotateCcw size={16} /></button>
      <button class="primary" onclick={main} title="{v.primary} (Space)">
        {#if v.status === 'running'}<Pause size={18} fill="currentColor" />{:else if v.primary === 'Start break'}<Coffee size={18} />{:else}<Play size={18} fill="currentColor" />{/if}
        {v.primary}
      </button>
      <button class="round" onclick={() => ipc.pomoSkip()} title="{v.skip} (S)" aria-label={v.skip}><SkipForward size={16} /></button>
    {/if}
  </div>

  <hr />
  <button class="today" onclick={() => (more = !more)} aria-expanded={more} title={more ? 'Hide the week' : 'Show the week'}>
    <span class="trow"><span class="k">Today</span><b>{v.today.text}</b></span>
    <span class="goal" role="progressbar" aria-label="Daily goal" aria-valuemin={0} aria-valuemax={v.today.goal} aria-valuenow={Math.min(v.today.sessions, v.today.goal)}>
      <span class="bar"><span style:width="{v.today.pct * 100}%"></span></span>
      <span class="gk">{v.today.sessions}/{v.today.goal}</span>
    </span>
  </button>
  {#if more && p.state}
    <div class="week">
      <div class="chart" role="img" aria-label="Focus minutes, last 7 days">
        {#each week as d, i (d.day)}
          <span class="col" title="{dayName(d.day, 'long')}: {fmtMinutes(d.minutes)}">
            <span class="b" class:today={i === week.length - 1} style:height="{d.minutes ? Math.max(3, (d.minutes / peak) * 44) : 2}px"></span>
            <span class="dl">{dayName(d.day, 'narrow')}</span>
          </span>
        {/each}
      </div>
      <div class="stats">
        <span>{p.state.streak ? `🔥 ${p.state.streak} ${p.state.streak === 1 ? 'day' : 'days'}` : 'No streak yet'}</span>
        <span class="k">Longest session <b>{p.state.longest ? fmtMinutes(p.state.longest) : '—'}</b></span>
      </div>
    </div>
  {/if}
</div>
{/if}

<style>
  .icon-btn.on { background: var(--pill-hover); }
  .pomo { display: flex; flex-direction: column; align-items: center; }
  :is(button):focus-visible { outline: 2px solid var(--c, var(--accent)); outline-offset: 2px; }

  /* Focus / Short / Long */
  .seg { display: flex; width: 100%; padding: 2px; gap: 2px; border-radius: var(--r-chip); background: var(--bg-2); }
  .seg button { flex: 1; height: 28px; border-radius: var(--r-chip); font-size: 12px; color: var(--fg-dim); transition: background 0.15s, color 0.15s; }
  .seg button:hover:not(.on) { color: var(--fg); }
  .seg button.on { background: var(--pill-hover); color: var(--fg); font-weight: 600; box-shadow: inset 0 -2px 0 var(--c); }

  .hero { margin: var(--s-4) 0 var(--s-3); }
  .time { font-size: 48px; font-weight: 600; line-height: 1; letter-spacing: -0.02em; font-variant-numeric: tabular-nums; color: var(--fg); transition: color 0.3s; }
  .time.urgent { color: var(--c); }
  .time.long { font-size: 36px; }
  .colon.blink { animation: blink 1s ease-in-out infinite alternate; }
  @keyframes blink { to { opacity: 0.4; } }
  .state { display: inline-flex; align-items: center; gap: 4px; margin-top: var(--s-2); font-size: 13px; color: var(--fg-dim); }

  /* Session dots: done = filled, current = half-filled (pulses while running), upcoming = outline. */
  .dots { display: flex; gap: var(--s-2); }
  .dots i { width: 10px; height: 10px; border-radius: 50%; box-shadow: inset 0 0 0 1.5px var(--fg-3); }
  .dots i.done { background: var(--c); box-shadow: none; }
  .dots i.current { background: linear-gradient(90deg, var(--c) 50%, transparent 50%); box-shadow: inset 0 0 0 1.5px var(--c); }
  .dots i.live { animation: pulse 2s ease-in-out infinite; }
  @keyframes pulse { 50% { transform: scale(1.2); } }
  .session { margin: var(--s-2) 0 var(--s-4); font-size: 12px; color: var(--fg-dim); }

  .controls { display: flex; align-items: center; justify-content: center; gap: var(--s-4); min-height: 44px; }
  .primary {
    display: inline-flex; align-items: center; justify-content: center; gap: var(--s-2); height: 44px; min-width: 156px; padding: 0 var(--s-6);
    border-radius: var(--r-chip); background: var(--c); color: var(--phase-ink); font-size: 14px; font-weight: 600;
    transition: filter 0.15s, transform 0.2s var(--spring);
  }
  .primary:hover { filter: brightness(1.08); }
  .primary:active { transform: scale(0.96); }
  .round {
    display: grid; place-items: center; width: 36px; height: 36px; border-radius: 50%; color: var(--fg);
    background: var(--pill); box-shadow: inset 0 0 0 1px var(--line-soft); transition: background 0.15s, transform 0.2s var(--spring);
  }
  .round:hover { background: var(--pill-hover); }
  .round:active { transform: scale(0.92); }
  .ask { font-weight: 600; }
  .chip { height: 32px; padding: 0 var(--s-4); border-radius: var(--r-chip); background: var(--pill); box-shadow: inset 0 0 0 1px var(--line-soft); }
  .chip:hover { background: var(--pill-hover); }
  .chip.yes { background: var(--c); color: var(--phase-ink); box-shadow: none; font-weight: 600; }

  hr { align-self: stretch; border: 0; height: 1px; background: var(--border); margin: var(--s-4) 0 var(--s-2); }
  .today { align-self: stretch; display: flex; flex-direction: column; gap: 6px; padding: var(--s-2); border-radius: var(--r-inner); text-align: left; }
  .today:hover { background: var(--bg-2); }
  .trow { display: flex; justify-content: space-between; align-items: baseline; }
  .k { color: var(--fg-dim); }
  .trow b { font-weight: 600; font-variant-numeric: tabular-nums; }
  .goal { display: flex; align-items: center; gap: var(--s-2); }
  .bar { flex: 1; height: 4px; border-radius: var(--r-chip); background: color-mix(in srgb, var(--fg) 10%, transparent); overflow: hidden; }
  .bar > span { display: block; height: 100%; border-radius: inherit; background: var(--c); transition: width 0.4s var(--ease); }
  .gk { font-size: 11px; color: var(--fg-dim); font-variant-numeric: tabular-nums; }

  .week { align-self: stretch; padding: var(--s-2); }
  .chart { display: flex; align-items: flex-end; justify-content: space-between; gap: var(--s-2); height: 64px; }
  .col { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: flex-end; gap: 4px; height: 100%; }
  .b { width: 100%; max-width: 22px; border-radius: calc(3px * var(--rs)); background: color-mix(in srgb, var(--fg) 28%, transparent); }
  .b.today { background: var(--c); }
  .dl { font-size: 11px; color: var(--fg-3); }
  .stats { display: flex; justify-content: space-between; margin-top: var(--s-3); font-size: 12px; }
  .stats b { color: var(--fg); font-weight: 600; }

  @media (prefers-reduced-motion: reduce) { .colon.blink, .dots i.live { animation: none; } .primary, .round { transition: none; } }
  :global(:root[data-motion='reduced']) :is(.colon.blink, .dots i.live) { animation: none; }
</style>
