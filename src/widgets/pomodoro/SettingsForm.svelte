<script lang="ts">
  // Pomodoro settings: shown by the flyout's gear and in Settings › Widgets.
  import { Minus, Plus } from '@lucide/svelte';
  import type { PomoCfg } from '../../lib/ipc';
  import { chime } from '../../lib/sound';
  import type { SettingsProps } from '../types';

  let { settings, save }: SettingsProps<PomoCfg> = $props();
  type Num = 'workMin' | 'shortMin' | 'longMin' | 'every' | 'goal';
  const STEPS: [Num, string, number, string][] = [
    ['workMin', 'Focus', 180, 'min'], ['shortMin', 'Short break', 60, 'min'], ['longMin', 'Long break', 120, 'min'],
    ['every', 'Sessions before long break', 12, ''], ['goal', 'Daily goal', 24, 'sessions'],
  ];
  // Minutes move by 5 (by 1 at 5 and below); counts by 1.
  const step = (k: Num, v: number, dir: 1 | -1) => (!k.endsWith('Min') || (dir < 0 ? v <= 5 : v < 5) ? 1 : 5) * dir;
  const set = (k: Num, max: number, v: number) => save({ [k]: Math.min(max, Math.max(1, Math.round(v))) });
  const val = (k: Num) => settings[k] ?? (k === 'goal' ? 4 : 1);
</script>

<div class="pset">
  {#each STEPS as [k, label, max, unit] (k)}
    {@const v = val(k)}
    <div class="row">
      <span>{label}</span>
      <span class="step">
        <button class="sbtn" disabled={v <= 1} onclick={() => set(k, max, v + step(k, v, -1))} aria-label="Less {label}" title="Less"><Minus size={14} /></button>
        <b>{v}{unit ? ` ${unit === 'sessions' && v === 1 ? 'session' : unit}` : ''}</b>
        <button class="sbtn" disabled={v >= max} onclick={() => set(k, max, v + step(k, v, 1))} aria-label="More {label}" title="More"><Plus size={14} /></button>
      </span>
    </div>
  {/each}
  <label class="row tall">
    <span class="txt">Auto-start next phase<small>When off, phases start only when you click.</small></span>
    <input type="checkbox" class="switch" checked={settings.autoStart} onchange={(e) => save({ autoStart: e.currentTarget.checked })} />
  </label>
  <label class="row">
    <span>Sound when a phase ends</span>
    <input type="checkbox" class="switch" checked={settings.sound ?? true} onchange={(e) => save({ sound: e.currentTarget.checked })} />
  </label>
  {#if settings.sound ?? true}
    <label class="row">
      <span>Volume</span>
      <input type="range" class="slider" min="0" max="100" step="5" value={settings.volume ?? 60} aria-label="Sound volume"
             onchange={(e) => { const v = +e.currentTarget.value; save({ volume: v }); chime(v / 100); }} />
    </label>
  {/if}
  <label class="row tall">
    <span class="txt">Dim other dock widgets while focusing<small>Everything but this widget fades during a focus session.</small></span>
    <input type="checkbox" class="switch" checked={settings.dimOthers} onchange={(e) => save({ dimOthers: e.currentTarget.checked })} />
  </label>
  <p class="note">When a phase ends you also get a banner, following your Notifications and Do Not Disturb settings.</p>
</div>

<style>
  .pset { display: flex; flex-direction: column; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: var(--s-3); min-height: 40px; padding: 0 var(--s-1); }
  .row.tall { padding: var(--s-1); }
  .row > span:first-child { color: var(--fg); }
  .txt { display: flex; flex-direction: column; gap: 2px; }
  small { font-size: 11px; color: var(--fg-dim); }
  .step { display: flex; align-items: center; gap: var(--s-1); }
  .step b { min-width: 72px; text-align: center; font-weight: 600; font-variant-numeric: tabular-nums; }
  .sbtn { display: grid; place-items: center; width: 28px; height: 28px; border-radius: 50%; color: var(--fg-dim); transition: background 0.15s, color 0.15s; }
  .sbtn:hover:not(:disabled) { background: var(--pill-hover); color: var(--fg); }
  .sbtn:active:not(:disabled) { transform: scale(0.92); }
  .sbtn:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .sbtn:disabled { opacity: 0.35; }
  .slider { width: 140px; }
  .note { margin: var(--s-2) var(--s-1) 0; font-size: 11px; color: var(--fg-dim); }
</style>
