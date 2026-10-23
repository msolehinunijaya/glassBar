<script lang="ts">
  import { MapPin } from '@lucide/svelte';
  import type { PopupProps } from '../types';
  import type { SolatSettings } from './index';
  import { countdown } from './next';
  import { liveSolat } from './solat.svelte';

  let { settings }: PopupProps<SolatSettings> = $props();
  const s = liveSolat(() => settings);
  const today = $derived(s.info?.days[0]);
  const at = (ms: number) => new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' }).format(ms);
  const shown = $derived(today?.times.filter(([n]) => settings.showImsak || n !== 'Imsak') ?? []);
</script>

<h3>Waktu Solat</h3>
{#if s.info && today}
  <div class="dim small loc">
    <MapPin size={12} />
    {s.info.zoneName ?? s.info.place?.name ?? 'Your location'}
    {#if today.source === 'aladhan'} · Aladhan (MWL){/if}
    {#if s.info.place?.source === 'ip' && !s.info.zoneName?.length} · Approximate location from your IP{/if}
  </div>
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
      <div class="row" class:next-row={s.next?.name === name && s.next.at === ms} class:past={ms < s.now}>
        <span>{name}</span><span class="t">{at(ms)}</span>
      </div>
    {/each}
  </div>
{:else if s.error}
  <p class="err">{s.error}</p>
{:else}
  <p class="dim empty">Loading prayer times…</p>
{/if}

<style>
  h3 { margin: 0 0 4px; font-size: 15px; font-weight: 600; }
  .small { font-size: 12px; }
  .loc { display: flex; align-items: center; gap: 4px; }
  .next { display: flex; align-items: baseline; gap: 8px; margin: 12px 0; padding: 12px 14px; }
  .next b { font-size: 18px; }
  .cd { margin-left: auto; font-variant-numeric: tabular-nums; font-weight: 500; }
  .row { display: flex; justify-content: space-between; padding: 9px 12px; border-radius: 10px; }
  .row.past { opacity: 0.5; }
  .row.next-row { background: var(--accent); color: #fff; font-weight: 600; opacity: 1; }
  .t { font-variant-numeric: tabular-nums; }
  .err { color: #ff453a; font-size: 12px; }
  .empty { text-align: center; padding: 60px 0; }
</style>
