<script lang="ts">
  // Three cards like the reference: day, month, year. Click a card to show it on the dock.
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import type { PopupProps } from '../types';
  import type { ProgressSettings } from './index';
  import DotGrid from './DotGrid.svelte';
  import { label, period, span, type Mode } from './period';

  let { instance, settings }: PopupProps<ProgressSettings> = $props();
  let now = $state(new Date());
  $effect(() => { const t = setInterval(() => (now = new Date()), 30_000); return () => clearInterval(t); });
  const modes: Mode[] = ['day', 'month', 'year'];
</script>

{#each modes as m (m)}
  {@const p = period(now, m)}
  <button class="card pcard" class:sel={settings.mode === m} onclick={() => updateWidgetSettings(instance, { mode: m })}
          title="{span(p.end - p.now)} left · click to show on the dock">
    <DotGrid mode={m} frac={p.frac} dot={m === 'year' ? 10 : 13} gap={m === 'year' ? 4 : 5} />
    <span class="foot"><span class="lbl">{label(now, m)}</span><b>{Math.floor(p.frac * 100)}%</b></span>
  </button>
{/each}

<style>
  .pcard { display: flex; flex-direction: column; gap: 9px; width: 100%; padding: 11px 13px 9px; margin-bottom: 7px; border-radius: calc(18px * var(--rs)); text-align: left; transition: box-shadow 0.2s, transform 0.25s var(--spring); }
  .pcard:hover { transform: translateY(-1px); }
  .pcard.sel { box-shadow: inset 0 0 0 1.5px color-mix(in srgb, var(--accent) 70%, transparent); }
  .foot { display: flex; justify-content: space-between; align-items: baseline; }
  .lbl { color: var(--fg-dim); font-size: 12px; }
  b { font-size: 14px; font-weight: 700; font-variant-numeric: tabular-nums; }
</style>
