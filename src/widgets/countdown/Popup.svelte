<script lang="ts">
  // Countdowns: a card each, soonest first. Click a card to put it on the dock (★); hover for
  // Edit and Delete. + adds one (from a preset or your own date).
  import { Pencil, Plus, Star, Trash2 } from '@lucide/svelte';
  import { flip } from 'svelte/animate';
  import PanelHeader from '../../lib/PanelHeader.svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import type { PopupProps } from '../types';
  import type { Countdown } from './dates';
  import Editor from './Editor.svelte';
  import type { CountdownSettings } from './index';
  import { countdownClock, keepRolled, views } from './live.svelte';
  import { PRESETS, fromPreset, isEstimate } from './presets';

  let { instance, settings }: PopupProps<CountdownSettings> = $props();
  const clock = countdownClock(() => settings.items ?? []);
  keepRolled(() => instance, () => settings, () => clock.now);
  const list = $derived(views(settings.items ?? [], clock.now));
  /** null = the list; 'new' or a countdown = the editor. */
  let editing = $state<Countdown | 'new' | null>(null);
  const DOTS = 20;
  const pinnedId = $derived(settings.pinned ?? list.find((v) => v.l.state !== 'done')?.c.id ?? null);

  const items = () => settings.items ?? [];
  function save(c: Countdown) {
    const has = items().some((x) => x.id === c.id);
    updateWidgetSettings(instance, { items: has ? items().map((x) => (x.id === c.id ? c : x)) : [...items(), c] });
    editing = null;
  }
  const remove = (id: string) => updateWidgetSettings(instance, { items: items().filter((x) => x.id !== id), pinned: settings.pinned === id ? null : settings.pinned });
  const pin = (id: string) => updateWidgetSettings(instance, { pinned: settings.pinned === id ? null : id });
  const quick = (p: (typeof PRESETS)[number]) => save(fromPreset(p, new Date()));
  const fmt = (d: Date, timed: boolean) => d.toLocaleDateString(undefined, { weekday: 'short', day: 'numeric', month: 'short', year: 'numeric' })
    + (timed ? ` · ${d.toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' })}` : '');
</script>

{#if editing}
  <Editor item={editing === 'new' ? null : editing} {save} cancel={() => (editing = null)} />
{:else}
  <PanelHeader title="Countdowns">
    <button class="icon-btn" onclick={() => (editing = 'new')} aria-label="New countdown" title="New countdown"><Plus size={17} /></button>
  </PanelHeader>

  {#each list as v (v.c.id)}
    {@const filled = Math.round(v.progress * DOTS)}
    <div class="card cd" class:today={v.l.state === 'today'} class:done={v.l.state === 'done'} animate:flip={{ duration: 200 }}>
      <button class="main" onclick={() => pin(v.c.id)} title={settings.pinned === v.c.id ? 'Shown on the dock · click to show the nearest instead' : 'Show this one on the dock'}>
        <span class="top">
          <span class="em">{v.c.emoji}</span>
          <span class="name ellipsis">{v.c.name}</span>
          {#if pinnedId === v.c.id}<Star size={13} class="star" fill={settings.pinned === v.c.id ? 'currentColor' : 'none'} />{/if}
        </span>
        <span class="date dim">{[fmt(v.target, !!v.c.time), isEstimate(v.c) ? 'estimate ±1 day' : '', v.c.repeat !== 'none' ? 'every year' : ''].filter(Boolean).join(' · ')}</span>
        <span class="hero">
          <b>{v.l.big}</b>
          <span class="dots" aria-hidden="true">{#each { length: DOTS } as _, i (i)}<i class:on={i < filled}></i>{/each}</span>
        </span>
        {#if v.l.small}<span class="small dim">{v.l.small}</span>{/if}
      </button>
      <span class="acts">
        <button class="ib" onclick={() => (editing = v.c)} aria-label="Edit {v.c.name}" title="Edit"><Pencil size={13} /></button>
        <button class="ib" onclick={() => remove(v.c.id)} aria-label="Delete {v.c.name}" title={v.l.state === 'done' ? 'Remove' : 'Delete'}><Trash2 size={13} /></button>
      </span>
    </div>
  {:else}
    <div class="empty">
      <p>Count down to something.</p>
      <div class="chips">
        {#each PRESETS.slice(0, 6) as p (p.key)}<button class="chip" onclick={() => quick(p)}><span>{p.emoji}</span>{p.name}</button>{/each}
      </div>
      <button class="own" onclick={() => (editing = 'new')}><Plus size={14} /> or make your own</button>
    </div>
  {/each}
{/if}

<style>
  .cd { position: relative; margin-bottom: var(--s-2); padding: 0; border-radius: var(--r-outer); transition: box-shadow 0.2s var(--ease); }
  .main { display: flex; flex-direction: column; gap: 4px; width: 100%; padding: 12px 14px; text-align: left; border-radius: inherit; }
  .main:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
  .top { display: flex; align-items: center; gap: 8px; padding-right: 56px; }
  .em { font-size: 18px; line-height: 1; }
  .name { flex: 1; min-width: 0; font-size: 14px; font-weight: 600; }
  .top :global(.star) { color: var(--accent); flex: none; margin-left: auto; }
  .name { display: block; }
  .date { font-size: 11.5px; }
  .hero { display: flex; align-items: center; justify-content: space-between; gap: 10px; margin-top: 4px; }
  .hero b { font-size: 20px; font-weight: 600; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .dots { display: grid; grid-template-columns: repeat(10, 6px); gap: 3px; }
  .dots i { width: 6px; height: 6px; border-radius: 50%; background: color-mix(in srgb, var(--fg) 14%, transparent); }
  .dots i.on { background: var(--accent); }
  .small { font-size: 11.5px; font-variant-numeric: tabular-nums; }
  .today { box-shadow: inset 0 0 0 1.5px color-mix(in srgb, var(--accent) 60%, transparent); }
  .today .dots i.on { background: #30d158; }
  .done { opacity: 0.55; }
  /* Edit / Delete appear on hover or keyboard focus. */
  .acts { position: absolute; top: 8px; right: 8px; display: flex; gap: 2px; opacity: 0; transition: opacity 0.15s; }
  .cd:hover .acts, .acts:focus-within { opacity: 1; }
  .ib { display: grid; place-items: center; width: 26px; height: 26px; border-radius: var(--r-inner); }
  .ib:hover { background: var(--pill-hover); }
  .ib:focus-visible { outline: 2px solid var(--accent); }
  .empty { text-align: center; padding: 16px 0 8px; }
  .empty p { margin: 0 0 12px; color: var(--fg-dim); font-size: 13px; }
  .chips { display: flex; flex-wrap: wrap; justify-content: center; gap: 6px; }
  .chip { display: inline-flex; align-items: center; gap: 5px; height: 28px; padding: 0 10px; border-radius: var(--r-chip); background: var(--pill); font-size: 12px; }
  .chip:hover { background: var(--pill-hover); }
  .own { display: inline-flex; align-items: center; gap: 4px; margin-top: 12px; color: var(--accent); font-size: 12px; }
</style>
