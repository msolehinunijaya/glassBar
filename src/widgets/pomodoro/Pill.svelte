<script lang="ts">
  // The pill is the progress bar. Every phase change needs a click: when a phase ends the
  // timer stops, the pill says so ("Work done ✓") and the play button starts the next phase.
  import { Check, Pause, Play, SkipForward } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import { clock } from '../../lib/time';
  import type { PillProps } from '../types';
  import { PHASE, stateOf } from './index';
  import { livePomo } from './pomo.svelte';

  let { size }: PillProps = $props();
  const p = livePomo();
  const phase = $derived(p.state ? PHASE[p.state.pomo.phase] : PHASE.work);
  const st = $derived(p.state ? stateOf(p.state.pomo) : null);
  const act = (f: () => Promise<unknown>) => (e: Event) => { e.stopPropagation(); f(); };
  const down = (e: PointerEvent) => e.stopPropagation();
</script>

<div class="pill pomo" class:done={st?.key === 'completed'} style:--c={phase.color} title="{st?.title ?? ''} · session {p.state?.label ?? ''}">
  <span class="fill" style:width="{st?.key === 'completed' ? 100 : Math.round(p.progress * 1000) / 10}%"></span>
  {#if st?.key === 'completed'}
    <Check size={14} strokeWidth={3} />
    <span class="t">{p.state?.pomo.completed === 'work' ? 'Work done' : 'Break done'}</span>
    {#if size !== 'small'}<span class="dim lbl">{st.action}</span>{/if}
  {:else}
    <span class="dot"></span>
    <span class="t">{clock(p.remaining)}</span>
    {#if size !== 'small' && p.state}<span class="dim lbl">{phase.name} {p.state.label}</span>{/if}
  {/if}
  <button class="ctl" onclick={act(() => (p.running ? ipc.pomoPause() : ipc.pomoStart()))} onpointerdown={down}
          aria-label={st?.action ?? 'Start'} title={st?.action}>
    {#if p.running}<Pause size={13} />{:else}<Play size={13} />{/if}
  </button>
  {#if st?.key !== 'completed'}
    <button class="ctl" onclick={act(ipc.pomoSkip)} onpointerdown={down} aria-label="Skip to next phase" title="Skip to next phase"><SkipForward size={13} /></button>
  {/if}
</div>

<style>
  .pomo { position: relative; overflow: hidden; gap: 6px; padding-right: 4px; }
  .fill {
    position: absolute; left: 0; top: 0; bottom: 0; z-index: 0;
    background: color-mix(in srgb, var(--c) 38%, transparent);
    border-right: 2px solid var(--c);
    transition: width 1s linear;
  }
  .done .fill { border-right: 0; background: color-mix(in srgb, var(--c) 55%, transparent); }
  .pomo > :not(.fill) { position: relative; z-index: 1; }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--c); }
  .t { font-variant-numeric: tabular-nums; font-weight: 600; }
  .lbl { font-size: 11px; }
  .ctl { display: grid; place-items: center; width: 24px; height: 24px; border-radius: 50%; transition: background 0.2s, transform 0.3s var(--spring); }
  .ctl:hover { background: var(--pill-hover); }
  .ctl:active { transform: scale(0.85); }
</style>
