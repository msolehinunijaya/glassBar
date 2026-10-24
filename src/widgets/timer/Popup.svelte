<script lang="ts">
  import { Flag, Pause, Play, RotateCcw, X } from '@lucide/svelte';
  import { ipc, type Stopwatch, type Timer } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { tickingNow } from '../../lib/now.svelte';
  import { clock } from '../../lib/time';
  import type { PopupProps } from '../types';

  let {}: PopupProps = $props();
  let tab = $state<'timers' | 'stopwatch'>('timers');
  const timers = live<Timer[]>('timers', ipc.timersList, []);
  const sw = live<Stopwatch | null>('stopwatch', ipc.swGet, null);
  const t = tickingNow(() => timers.v.some((x) => x.endsAt) || !!sw.v?.startedAt, 250);
  let custom = $state('');
  let label = $state('');
  let error = $state('');
  const presets = [1, 5, 10, 15, 30];

  async function add(ms: number) {
    try { await ipc.timerAdd(label, ms); label = ''; custom = ''; error = ''; } catch (e) { error = String(e); }
  }
  /** "90" = 90 min, "5:30" = 5 min 30 s, "1:00:00" = 1 h. */
  function parse(s: string): number | null {
    const parts = s.trim().split(':').map(Number);
    if (!parts.length || parts.some((n) => !Number.isFinite(n) || n < 0)) return null;
    if (parts.length === 1) return parts[0] * 60_000;
    return parts.reduce((acc, n) => acc * 60 + n, 0) * 1000;
  }
  const swElapsed = $derived(sw.v ? sw.v.accMs + (sw.v.startedAt ? t.now - sw.v.startedAt : 0) : 0);
  const left = (x: Timer) => (x.endsAt ? x.endsAt - t.now : x.remainingMs ?? 0);
  const done = (x: Timer) => x.endsAt === null && x.remainingMs === 0;
</script>

<div class="seg">
  <button class:sel={tab === 'timers'} onclick={() => (tab = 'timers')}>Timers</button>
  <button class:sel={tab === 'stopwatch'} onclick={() => (tab = 'stopwatch')}>Stopwatch</button>
</div>

{#if tab === 'timers'}
  <div class="presets">
    {#each presets as m (m)}<button class="tile" onclick={() => add(m * 60_000)}>{m} min</button>{/each}
  </div>
  <div class="custom card">
    <input placeholder="Label (optional)" bind:value={label} />
    <input class="dur" placeholder="mm:ss" bind:value={custom} onkeydown={(e) => e.key === 'Enter' && parse(custom) && add(parse(custom)!)} />
    <button class="btn" disabled={!parse(custom)} onclick={() => add(parse(custom)!)}>Start</button>
  </div>
  {#if error}<p class="err">{error}</p>{/if}
  <div class="list">
    {#each timers.v as x (x.id)}
      {@const ms = left(x)}
      <div class="row card" class:done={done(x)}>
        <div class="col">
          <span class="big">{done(x) ? 'Done' : clock(ms)}</span>
          <span class="dim small">{x.label || clock(x.durationMs)}</span>
        </div>
        {#if !done(x)}
          <button class="icon-btn" onclick={() => (x.endsAt ? ipc.timerPause(x.id) : ipc.timerResume(x.id))} aria-label={x.endsAt ? 'Pause' : 'Resume'}>
            {#if x.endsAt}<Pause size={16} />{:else}<Play size={16} />{/if}
          </button>
        {/if}
        <button class="icon-btn" onclick={() => ipc.timerDelete(x.id)} aria-label="Remove"><X size={16} /></button>
      </div>
    {/each}
  </div>
{:else}
  <div class="sw">
    <span class="huge">{clock(swElapsed)}<small>.{String(Math.floor((swElapsed % 1000) / 100))}</small></span>
    <div class="controls">
      <button class="icon-btn" onclick={() => ipc.swReset()} aria-label="Reset"><RotateCcw size={18} /></button>
      <button class="icon-btn play" onclick={() => ipc.swToggle()} aria-label={sw.v?.startedAt ? 'Stop' : 'Start'}>
        {#if sw.v?.startedAt}<Pause size={24} />{:else}<Play size={24} />{/if}
      </button>
      <button class="icon-btn" disabled={!sw.v?.startedAt} onclick={() => ipc.swLap()} aria-label="Lap"><Flag size={18} /></button>
    </div>
    <div class="laps">
      {#each [...(sw.v?.laps ?? [])].reverse() as lap, i (i)}
        {@const n = (sw.v?.laps.length ?? 0) - i}
        <div class="lap"><span class="dim">Lap {n}</span><span>{clock(lap)}</span></div>
      {/each}
    </div>
  </div>
{/if}

<style>
  .seg { display: grid; grid-template-columns: 1fr 1fr; gap: 2px; padding: 3px; border-radius: calc(12px * var(--rs)); background: var(--pill); margin-bottom: 12px; }
  .seg button { padding: 7px; border-radius: calc(9px * var(--rs)); }
  .seg .sel { background: var(--accent); color: #fff; }
  .presets { display: grid; grid-template-columns: repeat(5, 1fr); gap: 6px; }
  .presets .tile { align-items: center; padding: 10px 0; font-size: 12px; }
  .custom { display: flex; gap: 6px; margin-top: 8px; padding: 6px; }
  .custom input { flex: 1; min-width: 0; border: 0; background: none; outline: none; }
  .custom .dur { flex: 0 0 60px; }
  .list { display: flex; flex-direction: column; gap: 6px; margin-top: 10px; }
  .row { display: flex; align-items: center; gap: 6px; padding: 8px 10px; }
  .row.done { border-color: var(--accent); }
  .col { flex: 1; display: flex; flex-direction: column; }
  .big { font-size: 20px; font-weight: 600; font-variant-numeric: tabular-nums; }
  .small { font-size: 12px; }
  .sw { display: flex; flex-direction: column; align-items: center; gap: 14px; }
  .huge { font-size: 44px; font-weight: 600; font-variant-numeric: tabular-nums; }
  .huge small { font-size: 22px; opacity: 0.6; }
  .controls { display: flex; align-items: center; gap: 22px; }
  .play { width: 56px; height: 56px; background: var(--accent); color: #fff; }
  .laps { width: 100%; }
  .lap { display: flex; justify-content: space-between; padding: 6px 4px; border-bottom: 1px solid var(--line-soft); font-variant-numeric: tabular-nums; }
  .err { color: #ff453a; font-size: 12px; }
</style>
