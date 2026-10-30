<script lang="ts">
  // The pinned countdown (or the nearest): its emoji in a progress ring, and the time left.
  // On the day it glows softly; nothing flashes (it isn't urgent).
  import { Hourglass, Plus } from '@lucide/svelte';
  import type { PillProps } from '../types';
  import type { CountdownSettings } from './index';
  import { countdownClock, keepRolled, views } from './live.svelte';

  let { instance, size, settings, vertical }: PillProps<CountdownSettings> = $props();
  const clock = countdownClock(() => settings.items ?? []);
  keepRolled(() => instance, () => settings, () => clock.now);
  const all = $derived(views(settings.items ?? [], clock.now));
  const v = $derived(all.find((x) => x.c.id === settings.pinned) ?? all.find((x) => x.l.state !== 'done') ?? all[0]);
  const R = 12, C = 2 * Math.PI * R;
  const tip = $derived(v ? `${v.c.name} · ${v.l.big}${v.l.small ? ` (${v.l.small})` : ''} · ${v.target.toLocaleDateString(undefined, { weekday: 'short', day: 'numeric', month: 'short', year: 'numeric' })}` : 'Countdowns · click to add one');
</script>

{#snippet ring()}
  <span class="ring" aria-hidden="true">
    <svg viewBox="0 0 28 28" width="28" height="28">
      <circle cx="14" cy="14" r={R} class="track" />
      <circle cx="14" cy="14" r={R} class="arc" stroke-dasharray={C} stroke-dashoffset={C * (1 - (v?.progress ?? 0))} />
    </svg>
    <span class="em">{v?.c.emoji}</span>
  </span>
{/snippet}

{#if vertical}
  <div class="side cd" class:today={v?.l.state === 'today'} class:done={v?.l.state === 'done'} title={tip}>
    {#if v}{@render ring()}<span class="side-v">{v.l.short}</span>{:else}<Hourglass size={18} /><span class="side-c">Add</span>{/if}
  </div>
{:else}
  <div class="pill cd" class:today={v?.l.state === 'today'} class:done={v?.l.state === 'done'} title={tip}>
    {#if v}
      {@render ring()}
      {#if size === 'small'}
        <b class="short">{v.l.short}</b>
      {:else}
        <span class="stack"><span class="name ellipsis">{v.c.name}</span><span class="t">{v.l.big}</span></span>
      {/if}
    {:else}
      <Hourglass size={16} /><span class="dim add"><Plus size={12} /> Add</span>
    {/if}
  </div>
{/if}

<style>
  .cd { gap: 7px; padding: 0 10px 0 5px; transition: box-shadow 0.4s var(--ease), background 0.4s var(--ease); }
  .ring { position: relative; display: grid; place-items: center; width: 28px; height: 28px; flex: none; }
  .ring svg { position: absolute; inset: 0; transform: rotate(-90deg); }
  .track { fill: none; stroke: var(--track); stroke-width: 2.4; }
  .arc { fill: none; stroke: var(--accent); stroke-width: 2.4; stroke-linecap: round; transition: stroke-dashoffset 0.6s var(--ease); }
  .em { font-size: 13px; line-height: 1; }
  .stack { display: flex; flex-direction: column; line-height: 1.15; min-width: 0; max-width: 120px; }
  .name { font-size: 10.5px; color: var(--fg-dim); }
  .t, .short { font-size: 12.5px; font-weight: 600; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .add { display: inline-flex; align-items: center; gap: 2px; font-size: 12px; }
  /* On the day: a calm accent glow. */
  .today { background: color-mix(in srgb, var(--accent) 22%, var(--pill)); box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 50%, transparent), 0 0 12px color-mix(in srgb, var(--accent) 30%, transparent); }
  .today .arc { stroke: #30d158; }
  .done { opacity: 0.6; }
</style>
