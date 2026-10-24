<script lang="ts">
  import { ipc, type ScreenToday } from '../../lib/ipc';
  import { duration } from '../../lib/time';
  import type { PopupProps } from '../types';
  import type { ScreenTimeSettings } from './index';
  import { updateWidgetSettings } from '../../lib/config.svelte';

  let { instance, settings }: PopupProps<ScreenTimeSettings> = $props();
  let today = $state<ScreenToday | null>(null);
  let week = $state<[string, number][]>([]);
  let icons = $state<Record<string, string | null>>({});
  const requested = new Set<string>();

  $effect(() => {
    ipc.screentimeToday().then((t) => {
      today = t;
      for (const a of t.apps) if (!requested.has(a.exe)) { requested.add(a.exe); ipc.appIcon(a.exe).then((i) => (icons[a.exe] = i)); }
    });
    ipc.screentimeWeek().then((w) => (week = w));
  });

  const top = $derived(Math.max(1, ...(today?.apps.map((a) => a.seconds) ?? [1])));
  const weekMax = $derived(Math.max(1, ...week.map((d) => d[1])));
  const dow = (d: string) => new Intl.DateTimeFormat(undefined, { weekday: 'narrow' }).format(new Date(d + 'T12:00'));
</script>

<h3>Screen time</h3>
<div class="total">{duration(today?.total ?? 0)} <span class="dim">today</span></div>

<div class="week">
  {#each week as [d, s], i (d)}
    <div class="col" title="{d}: {duration(s)}">
      <div class="bar" class:today={i === week.length - 1} style:height="{(s / weekMax) * 100}%"></div>
      <span class="dim">{dow(d)}</span>
    </div>
  {/each}
</div>

<div class="apps">
  {#each today?.apps ?? [] as a (a.exe)}
    <div class="app">
      {#if icons[a.exe]}<img src={icons[a.exe]} alt="" width="20" height="20" />{:else}<span class="ph"></span>{/if}
      <div class="info">
        <div class="row"><span class="ellipsis">{a.name}</span><span class="dim">{duration(a.seconds)}</span></div>
        <div class="track"><div style:width="{(a.seconds / top) * 100}%"></div></div>
      </div>
    </div>
  {:else}
    <p class="dim empty">No usage recorded yet today.</p>
  {/each}
</div>
<label class="dim gsw">7-day graph on the dock
  <input type="checkbox" class="switch" checked={settings.showGraph} onchange={(e) => updateWidgetSettings(instance, { showGraph: e.currentTarget.checked })} />
</label>
<p class="dim note">Recorded only while this widget is on your bar. Kept for 30 days, on this PC only.</p>

<style>
  h3 { margin: 0 0 4px; font-size: 15px; font-weight: 600; }
  .gsw { display: flex; align-items: center; justify-content: space-between; font-size: 12px; margin: 10px 2px 0; }
  .total { font-size: 26px; font-weight: 600; }
  .total .dim { font-size: 13px; font-weight: 400; }
  .week { display: grid; grid-template-columns: repeat(7, 1fr); gap: 6px; height: 90px; margin: 12px 0; align-items: end; }
  .col { display: flex; flex-direction: column; align-items: center; gap: 4px; height: 100%; justify-content: flex-end; font-size: 10px; }
  .bar { width: 100%; min-height: 2px; border-radius: calc(4px * var(--rs)); background: var(--track); }
  .bar.today { background: var(--accent); }
  .apps { display: flex; flex-direction: column; gap: 8px; }
  .app { display: flex; align-items: center; gap: 10px; }
  .ph { width: 20px; height: 20px; border-radius: calc(5px * var(--rs)); background: var(--track); }
  .info { flex: 1; min-width: 0; }
  .row { display: flex; justify-content: space-between; gap: 8px; font-size: 12px; }
  .track { height: 5px; border-radius: calc(3px * var(--rs)); background: var(--track); margin-top: 3px; }
  .track div { height: 100%; border-radius: calc(3px * var(--rs)); background: var(--accent); }
  .empty { text-align: center; padding: 20px 0; }
  .note { font-size: 11px; margin-top: 12px; }
</style>
