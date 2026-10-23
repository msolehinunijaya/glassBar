<script lang="ts">
  // The pill itself is the progress bar: its background fills left to right in the
  // phase colour. Pause/resume and skip work without opening the popup.
  import { Pause, Play, SkipForward } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import { clock } from '../../lib/time';
  import type { PillProps } from '../types';
  import { PHASE } from './index';
  import { livePomo } from './pomo.svelte';

  let { size }: PillProps = $props();
  const p = livePomo();
  const phase = $derived(p.state ? PHASE[p.state.pomo.phase] : PHASE.work);
  const act = (f: () => Promise<unknown>) => (e: Event) => { e.stopPropagation(); f(); };
</script>

<div class="pill pomo" style:--c={phase.color} title="{phase.name} · session {p.state?.label ?? ''}">
  <span class="fill" style:width="{Math.round(p.progress * 1000) / 10}%"></span>
  <span class="dot"></span>
  <span class="t">{clock(p.remaining)}</span>
  {#if size !== 'small' && p.state}<span class="dim lbl">{phase.name} {p.state.label}</span>{/if}
  <button class="ctl" onclick={act(() => (p.running ? ipc.pomoPause() : ipc.pomoStart()))} onpointerdown={(e) => e.stopPropagation()}
          aria-label={p.running ? 'Pause' : 'Start'}>
    {#if p.running}<Pause size={13} />{:else}<Play size={13} />{/if}
  </button>
  <button class="ctl" onclick={act(ipc.pomoSkip)} onpointerdown={(e) => e.stopPropagation()} aria-label="Next phase"><SkipForward size={13} /></button>
</div>

<style>
  .pomo { position: relative; overflow: hidden; gap: 6px; padding-right: 4px; }
  .fill {
    position: absolute; left: 0; top: 0; bottom: 0; z-index: 0;
    background: color-mix(in srgb, var(--c) 38%, transparent);
    border-right: 2px solid var(--c);
    transition: width 1s linear;
  }
  .pomo > :not(.fill) { position: relative; z-index: 1; }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--c); }
  .t { font-variant-numeric: tabular-nums; font-weight: 600; }
  .lbl { font-size: 11px; }
  .ctl { display: grid; place-items: center; width: 24px; height: 24px; border-radius: 50%; transition: background 0.2s, transform 0.3s var(--spring); }
  .ctl:hover { background: var(--pill-hover); }
  .ctl:active { transform: scale(0.85); }
</style>
