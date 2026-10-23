<script lang="ts">
  import { Pause, Play, RotateCcw, SkipForward } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import Ring from '../../lib/Ring.svelte';
  import { clock } from '../../lib/time';
  import type { PopupProps } from '../types';
  import { PHASE } from './index';
  import { livePomo } from './pomo.svelte';

  let {}: PopupProps = $props();
  const p = livePomo();
  const phase = $derived(p.state ? PHASE[p.state.pomo.phase] : PHASE.work);
</script>

<div class="wrap">
  <span class="phase" style:color={phase.color}>{phase.name}</span>
  <Ring value={p.progress} size={190} stroke={10} color={phase.color}>
    <span class="big">{clock(p.remaining)}</span>
    <span class="dim">Session {p.state?.label}</span>
  </Ring>
  <div class="controls">
    <button class="icon-btn" onclick={() => ipc.pomoReset()} aria-label="Reset"><RotateCcw size={18} /></button>
    <button class="icon-btn play" style:background={phase.color} onclick={() => (p.running ? ipc.pomoPause() : ipc.pomoStart())} aria-label={p.running ? 'Pause' : 'Start'}>
      {#if p.running}<Pause size={24} />{:else}<Play size={24} />{/if}
    </button>
    <button class="icon-btn" onclick={() => ipc.pomoSkip()} aria-label="Skip"><SkipForward size={18} /></button>
  </div>
</div>

<style>
  .wrap { display: flex; flex-direction: column; align-items: center; gap: 12px; padding-top: 6px; }
  .phase { font-weight: 600; font-size: 15px; }
  .big { font-size: 40px; font-weight: 600; font-variant-numeric: tabular-nums; line-height: 1; }
  .controls { display: flex; align-items: center; gap: 22px; }
  .play { width: 56px; height: 56px; color: #fff; }
</style>
