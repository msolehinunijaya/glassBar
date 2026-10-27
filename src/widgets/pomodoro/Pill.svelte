<script lang="ts">
  // Pomodoro on the dock. Small = a mini ring in the phase colour; Medium adds the time;
  // Large adds "Focus · 3/4". Paused = dashed ring. Never started today = a plain tomato.
  // Hovering turns the ring into ⏯ (pause/resume without opening the flyout).
  import { Pause, Play } from '@lucide/svelte';
  import { clock } from '../../lib/time';
  import type { PillProps } from '../types';
  import { PHASE } from './index';
  import PhaseRing from './PhaseRing.svelte';
  import { livePomo, toggle } from './pomo.svelte';
  import Tomato from './Tomato.svelte';

  let { size, vertical }: PillProps = $props();
  const p = livePomo();
  const v = $derived(p.view);
  const color = $derived(PHASE[v?.phase ?? 'work'].color);
  const paused = $derived(v?.status === 'paused');
  const tip = $derived(v ? (v.idle ? 'Pomodoro · click to start focusing' : `${PHASE[v.phase].name} · ${v.label} · ${clock(p.remaining)} left · session ${v.count}`) : 'Pomodoro');
  const act = $derived(v?.status === 'running' ? 'Pause' : v?.status === 'paused' ? 'Resume' : (v?.primary ?? 'Start focus'));
  // Side tile: whole minutes inside the ring ("0:42" in the last minute).
  const mins = $derived(p.remaining >= 60_000 ? String(Math.ceil(p.remaining / 60_000)) : clock(p.remaining));
  const quick = (e: Event) => { e.stopPropagation(); toggle(p.running); };
  const down = (e: PointerEvent) => e.stopPropagation();
</script>

{#snippet qt(sz: number)}
  <button class="qt" onclick={quick} onpointerdown={down} aria-label={act} title={act}>
    {#if p.running}<Pause size={sz} fill="currentColor" />{:else}<Play size={sz} fill="currentColor" />{/if}
  </button>
{/snippet}

{#if vertical}
<div class="side pomo-side" style:--c={color} title={tip}>
  <span class="slot">
    {#if !v || v.idle}<Tomato size={30} />
    {:else}
      <PhaseRing size={40} stroke={3} left={p.left} {paused} ticking={p.running} glow={3}>
        <span class="mid" class:urgent={v.status === 'running' && p.remaining <= 60_000}>{mins}</span>
      </PhaseRing>
    {/if}
    {@render qt(14)}
  </span>
  <span class="side-c">{!v || v.idle ? 'Focus' : v.phase === 'work' ? 'Focus' : 'Break'}</span>
</div>
{:else}
<div class="pill pomo" style:--c={color} title={tip}>
  <span class="slot">
    {#if !v || v.idle}<Tomato size={18} />
    {:else}<PhaseRing size={20} stroke={3} left={p.left} {paused} ticking={p.running} glow={3} />{/if}
    {@render qt(10)}
  </span>
  {#if v && !v.idle && size !== 'small'}
    <span class="stack">
      <span class="t" class:urgent={v.status === 'running' && p.remaining <= 60_000}>{clock(p.remaining)}</span>
      {#if size === 'large'}<span class="lbl">{PHASE[v.phase].name} · {v.count}</span>{/if}
    </span>
  {/if}
</div>
{/if}

<style>
  .pomo { gap: 8px; }
  .slot { position: relative; display: grid; place-items: center; }
  .slot > :global(*) { grid-area: 1 / 1; }
  .qt {
    /* Above the ring: it's positioned, so it sat on top and took the click (which then
       opened the panel instead of starting or pausing). */
    position: relative; z-index: 1;
    display: grid; place-items: center; width: 100%; height: 100%; border-radius: 50%;
    background: var(--c); color: var(--phase-ink); opacity: 0; transform: scale(0.8);
    transition: opacity 0.15s, transform 0.2s var(--spring);
  }
  .pill:hover .qt, .side:hover .qt, .qt:focus-visible { opacity: 1; transform: none; }
  .qt:active { transform: scale(0.88); }
  .t { font-weight: 600; font-variant-numeric: tabular-nums; }
  .urgent { color: var(--c); }
  .stack { display: flex; flex-direction: column; line-height: 1.15; min-width: 0; }
  .lbl { font-size: 10.5px; color: var(--fg-dim); white-space: nowrap; }
  .pomo-side { gap: 2px; padding-bottom: 4px; }
  .pomo-side .qt { width: 30px; height: 30px; }
  .mid { font-weight: 600; font-size: 12px; font-variant-numeric: tabular-nums; }
  @media (prefers-reduced-motion: reduce) { .qt { transition: none; } }
</style>
