<script lang="ts">
  // Tasbih on the dock, joined to the prayer pill: click = +1, wheel down = undo, right-click
  // for reset, target and dhikr. Reaching the target: a soft tick, a green pulse and ✓; the
  // next click starts the next round. Doesn't open the prayer panel.
  import { Check } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc, type TasbihState } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { showMenu } from '../../lib/menu';
  import { tick } from '../../lib/sound';
  import type { SolatSettings } from './index';
  import { DHIKR, TARGETS, phraseOf, stepsOf } from './tasbih';

  let { instance, settings, vertical = false, open }: { instance: string; settings: SolatSettings; vertical?: boolean; open: (el: HTMLElement) => void } = $props();
  const t = live<TasbihState>('tasbih', ipc.tasbihGet, { count: 0, round: 0, step: 0, today: 0, day: '' });
  const steps = $derived(stepsOf(settings.dhikr ?? 'subhanallah', settings.tasbihTarget ?? 33));
  const target = $derived(steps[Math.min(t.v.step, steps.length - 1)] ?? 33);
  const done = $derived(t.v.count >= target);
  const R = 10, C = 2 * Math.PI * R;
  let pop = $state(false);
  let el = $state<HTMLElement>();

  async function add(e: MouseEvent) {
    e.stopPropagation();
    const before = done;
    const s = await ipc.tasbihAdd(1, steps).catch(() => null);
    if (!s) return;
    t.v = s;
    pop = false; void el?.offsetWidth; pop = true;
    const tgt = steps[Math.min(s.step, steps.length - 1)];
    if (!before && s.count >= tgt) tick();
  }
  async function undo() { const s = await ipc.tasbihAdd(-1, steps).catch(() => null); if (s) t.v = s; }
  function wheel(e: WheelEvent) { if (e.deltaY > 0) { e.preventDefault(); e.stopPropagation(); undo(); } }
  async function menu(e: MouseEvent) {
    e.preventDefault(); e.stopPropagation();
    await showMenu([
      { text: 'Undo (−1)', enabled: t.v.count > 0, action: undo },
      { text: 'Reset', action: () => ipc.tasbihReset().then((s) => (t.v = s)) },
      '-',
      { text: 'Target', enabled: settings.dhikr !== 'tasbih_fatimah', items: TARGETS.map((n) => ({ text: n.toLocaleString(), checked: (settings.tasbihTarget ?? 33) === n, action: () => updateWidgetSettings(instance, { tasbihTarget: n }) })) },
      { text: 'Dhikr', items: DHIKR.map((d) => ({ text: d.label, checked: (settings.dhikr ?? 'subhanallah') === d.key, action: () => updateWidgetSettings(instance, { dhikr: d.key }) })) },
      '-',
      { text: 'Open Tasbih', action: () => el && open(el) },
    ]);
  }
  const tip = $derived(`${phraseOf(settings.dhikr ?? 'subhanallah', t.v.step)} · ${t.v.count} of ${target}${t.v.round ? ` · round ${t.v.round + 1}` : ''} · today ${t.v.today}\nClick +1 · scroll down to undo · right-click for more`);
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class={vertical ? 'side tas' : 'pill tas'} class:done class:pop bind:this={el} role="button" tabindex="-1" title={tip} aria-label="Tasbih {t.v.count} of {target}, click to count"
     onpointerdown={(e) => e.stopPropagation()} onclick={add} oncontextmenu={menu} onwheel={wheel} onanimationend={() => (pop = false)}>
  <span class="ring" aria-hidden="true">
    <svg viewBox="0 0 24 24" width="24" height="24">
      <circle cx="12" cy="12" r={R} class="track" />
      <circle cx="12" cy="12" r={R} class="arc" stroke-dasharray={C} stroke-dashoffset={C * (1 - Math.min(1, t.v.count / target))} />
    </svg>
    {#if done}<Check size={12} strokeWidth={3} />{:else}<i class="bead"></i>{/if}
  </span>
  {#if vertical}
    <span class="side-v">{t.v.count}</span>
  {:else}
    <span class="n"><b>{t.v.count}</b><small>/{target}</small></span>
  {/if}
</div>

<style>
  .tas { gap: 6px; padding: 0 10px 0 6px; cursor: pointer; user-select: none; transition: background 0.3s var(--ease), box-shadow 0.3s var(--ease); }
  .tas:hover { background: var(--pill-hover); }
  .ring { position: relative; display: grid; place-items: center; width: 24px; height: 24px; flex: none; }
  .ring svg { position: absolute; inset: 0; transform: rotate(-90deg); }
  .track { fill: none; stroke: var(--track); stroke-width: 2.4; }
  .arc { fill: none; stroke: var(--accent); stroke-width: 2.4; stroke-linecap: round; transition: stroke-dashoffset 0.25s var(--ease); }
  .bead { width: 6px; height: 6px; border-radius: 50%; background: var(--fg-dim); }
  .n { font-variant-numeric: tabular-nums; white-space: nowrap; line-height: 1; }
  .n b { font-size: 13px; font-weight: 600; }
  .n small { font-size: 10.5px; color: var(--fg-dim); }
  .done { background: color-mix(in srgb, #30d158 24%, var(--pill)); }
  .done .arc { stroke: #30d158; }
  .done .ring { color: #30d158; }
  .pop .ring { animation: bump 0.12s var(--ease); }
  .done.pop { animation: glow 0.6s var(--ease); }
  @keyframes bump { 50% { transform: scale(0.86); } }
  @keyframes glow { 40% { box-shadow: 0 0 0 2px #30d158, 0 0 14px rgba(48, 209, 88, 0.6); } }
  @media (prefers-reduced-motion: reduce) { .pop .ring, .done.pop { animation: none; } }
</style>
