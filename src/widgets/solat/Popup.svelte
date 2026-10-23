<script lang="ts">
  import { MapPin, Play, Settings2 } from '@lucide/svelte';
  import PanelHeader from '../../lib/PanelHeader.svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc } from '../../lib/ipc';
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { PopupProps } from '../types';
  import type { SolatSettings } from './index';
  import { countdown } from './next';
  import { liveSolat } from './solat.svelte';
  import LocationTag from '../../lib/LocationTag.svelte';

  let { instance, settings }: PopupProps<SolatSettings> = $props();
  let showOpts = $state(false);
  const s = liveSolat(() => settings);
  const today = $derived(s.info?.days[0]);
  // "05:51" + a small "AM" (12-hour locales), or "05:51" in 24-hour.
  const at = (ms: number) => {
    const parts = new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' }).formatToParts(ms);
    return { main: parts.filter((p) => p.type !== 'dayPeriod').map((p) => p.value).join('').trim(), ap: parts.find((p) => p.type === 'dayPeriod')?.value.toUpperCase() ?? '' };
  };
  const shown = $derived(today?.times.filter(([n]) => settings.showImsak || n !== 'Imsak') ?? []);
</script>

<PanelHeader title="Waktu Solat">
  <button class="icon-btn" class:on={showOpts} onclick={() => (showOpts = !showOpts)} aria-label="Prayer alert and adzan options" title="Alert and adzan sound"><Settings2 size={16} /></button>
</PanelHeader>
{#if showOpts}
  <OptionsPopover title="At prayer time" values={settings} save={(p) => updateWidgetSettings(instance, p)} options={[
    { type: 'toggle', key: 'alerts', label: 'Alert at each prayer time' },
    { type: 'choice', key: 'azanSound', label: 'Sound', menu: true, choices: [
      { value: 'short', label: 'Short adzan', hint: 'The opening takbir, about 40 seconds' },
      { value: 'full', label: 'Full adzan' },
      { value: 'chime', label: 'Normal alert sound', hint: 'The usual short alert sound' }] },
  ]} />
  <button class="btn test" disabled={!settings.alerts} onclick={() => ipc.solatTestSound(settings)}><Play size={12} /> Test sound</button>
  <p class="dim small hint">Press ✓ on the prayer pill to stop the adzan. Your own recording can be set in Settings › Widgets.</p>
{/if}
{#if s.info && today}
  <div class="dim small loc">
    <MapPin size={12} />
    {s.info.zoneName ?? s.info.place?.name ?? 'Your location'}
    {#if today.source === 'aladhan'} · Aladhan (MWL){/if}
  </div>
  {#if settings.zone === 'auto'}<LocationTag />{/if}
  <div class="dim small">
    {new Intl.DateTimeFormat(undefined, { dateStyle: 'full' }).format(new Date(today.date + 'T12:00'))}
    {#if today.hijri} · {today.hijri} H{/if}
  </div>
  {#if s.next}
    <div class="next card">
      <span class="dim">Next</span>
      <b>{s.next.name}</b>
      <span class="cd">in {countdown(s.next.at - s.now)}</span>
    </div>
  {/if}
  <div class="list">
    {#each shown as [name, ms] (name)}
      {@const t = at(ms)}
      <div class="row" class:next-row={s.next?.name === name && s.next.at === ms} class:past={ms < s.now}>
        <span>{name}</span><span class="t">{t.main}{#if t.ap}<small class="ap">{t.ap}</small>{/if}</span>
      </div>
    {/each}
  </div>
{:else if s.error}
  <p class="err">{s.error}</p>
  {#if settings.zone === 'auto'}<LocationTag />{/if}
{:else}
  <p class="dim empty">Loading prayer times…</p>
{/if}

<style>
  .icon-btn.on { color: var(--accent); }
  .test { display: inline-flex; align-items: center; gap: 6px; margin: 8px 0 4px; padding: 6px 12px; border-radius: 10px; background: var(--pill); font-size: 12px; }
  .test:hover:not(:disabled) { background: var(--pill-hover); }
  .test:disabled { opacity: 0.4; }
  .hint { margin: 4px 0 10px; }
  .small { font-size: 12px; }
  .loc { display: flex; align-items: center; gap: 4px; }
  .next { display: flex; align-items: baseline; gap: 8px; margin: 12px 0; padding: 12px 14px; }
  .next b { font-size: 18px; }
  .cd { margin-left: auto; font-variant-numeric: tabular-nums; font-weight: 500; }
  /* 40 px rows; the next prayer gets the inset surface and a 3 px accent bar, past ones fade to tertiary. */
  .row { display: flex; align-items: center; justify-content: space-between; min-height: 40px; padding: 0 var(--s-3); border-radius: var(--r-inner); }
  .row.past { color: var(--fg-3); }
  .row.next-row { background: var(--bg-2); box-shadow: inset 3px 0 0 var(--accent); font-weight: 600; color: var(--fg); }
  .ap { margin-left: 3px; font-size: 11px; font-weight: 500; color: var(--fg-3); }
  .t { font-variant-numeric: tabular-nums; }
  .err { color: #ff453a; font-size: 12px; }
  .empty { text-align: center; padding: 60px 0; }
</style>
