<script lang="ts">
  import { Pause, Play, RotateCcw, SkipForward } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import Ring from '../../lib/Ring.svelte';
  import { clock } from '../../lib/time';
  import type { PopupProps } from '../types';
  import { PHASE, stateOf } from './index';
  import { livePomo } from './pomo.svelte';

  let {}: PopupProps = $props();
  const p = livePomo();
  const phase = $derived(p.state ? PHASE[p.state.pomo.phase] : PHASE.work);
  const st = $derived(p.state ? stateOf(p.state.pomo) : null);
  const main = () => (p.running ? ipc.pomoPause() : ipc.pomoStart());
</script>

<div class="wrap">
  <span class="state" style:color={phase.color}>{st?.title ?? ''}</span>
  <Ring value={st?.key === 'completed' ? 1 : p.progress} size={180} stroke={10} color={phase.color}>
    <span class="big">{st?.key === 'completed' ? '✓' : clock(p.remaining)}</span>
    <span class="dim">{phase.name} · session {p.state?.label}</span>
  </Ring>
  <button class="primary" style:background={phase.color} onclick={main}>
    {#if p.running}<Pause size={18} />{:else}<Play size={18} />{/if}
    {st?.action ?? 'Start'}
  </button>
  <div class="row">
    <button class="icon-btn" onclick={() => ipc.pomoReset()} title="Reset" aria-label="Reset"><RotateCcw size={16} /></button>
    {#if st?.key !== 'completed'}
      <button class="icon-btn" onclick={() => ipc.pomoSkip()} title="Skip to next phase (it won't start until you click)" aria-label="Skip phase"><SkipForward size={16} /></button>
    {/if}
  </div>
  <p class="dim note">Each phase waits for your click: nothing starts by itself.</p>
</div>

<style>
  .wrap { display: flex; flex-direction: column; align-items: center; gap: 10px; padding-top: 4px; }
  .state { font-weight: 600; font-size: 15px; }
  .big { font-size: 38px; font-weight: 600; font-variant-numeric: tabular-nums; line-height: 1; }
  .primary { display: inline-flex; align-items: center; gap: 8px; padding: 9px 20px; border-radius: 20px; color: #fff; font-weight: 600;
             transition: transform 0.25s var(--spring), filter 0.2s; }
  .primary:hover { filter: brightness(1.08); }
  .primary:active { transform: scale(0.95); }
  .row { display: flex; gap: 14px; }
  .note { font-size: 11px; margin: 0; }
</style>
