<script lang="ts">
  // Add or edit a countdown: start from a preset (fills everything), or name a date yourself.
  import { ChevronLeft } from '@lucide/svelte';
  import { parseYmd, ymd, type Countdown } from './dates';
  import { EMOJI, PRESETS, fromPreset, isEstimate } from './presets';

  let { item, save, cancel }: { item: Countdown | null; save: (c: Countdown) => void; cancel: () => void } = $props();
  const now = new Date();
  const tomorrow = ymd(new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1));
  // svelte-ignore state_referenced_locally
  let c = $state<Countdown>(item ? { ...item } : { id: `c-${Date.now().toString(36)}`, name: '', date: tomorrow, time: null, emoji: '🎯', repeat: 'none', rule: null, created: Date.now(), remindDayBefore: false });
  let nameEl = $state<HTMLInputElement>();
  $effect(() => { nameEl?.focus(); });
  const valid = $derived(c.name.trim().length > 0 && /^\d{4}-\d{2}-\d{2}$/.test(c.date));
  const pick = (p: (typeof PRESETS)[number]) => { const id = c.id; c = { ...fromPreset(p, now), id, created: item?.created ?? Date.now() }; };
  const submit = (e: Event) => { e.preventDefault(); if (valid) save({ ...c, name: c.name.trim() }); };
  // Changing the date by hand makes it a plain date (or a yearly one): no calendar rule.
  const setDate = (v: string) => { c.date = v; if (c.repeat === 'hijri' || c.repeat === 'chinese') { c.repeat = 'none'; c.rule = null; } };
</script>

<div class="head">
  <button class="icon-btn" onclick={cancel} aria-label="Back"><ChevronLeft size={16} /></button>
  <h3>{item ? 'Edit countdown' : 'New countdown'}</h3>
</div>

{#if !item}
  <h4 class="dim">Start from</h4>
  <div class="chips">
    {#each PRESETS as p (p.key)}
      <button class="chip" class:sel={c.name === p.name} onclick={() => pick(p)}><span>{p.emoji}</span>{p.name}</button>
    {/each}
  </div>
{/if}

<form onsubmit={submit}>
  <label class="field"><span class="dim">Name</span>
    <input bind:this={nameEl} bind:value={c.name} placeholder="Project deadline" maxlength="40" />
  </label>
  <div class="two">
    <label class="field"><span class="dim">Date</span>
      <input type="date" value={c.date} min={ymd(now)} onchange={(e) => setDate(e.currentTarget.value)} required />
    </label>
    <label class="field"><span class="dim">Time (optional)</span>
      <input type="time" value={c.time ?? ''} onchange={(e) => (c.time = e.currentTarget.value || null)} />
    </label>
  </div>
  {#if isEstimate(c)}
    <p class="dim small">Calculated date ({c.repeat === 'hijri' ? 'Umm al-Qura' : 'Chinese calendar'}), may differ by a day. Moves to next year by itself.</p>
  {/if}
  <div class="field"><span class="dim">Icon</span>
    <div class="emoji" role="radiogroup" aria-label="Icon">
      {#each [...new Set([c.emoji, ...EMOJI])] as e (e)}
        <button type="button" role="radio" aria-checked={c.emoji === e} class:sel={c.emoji === e} onclick={() => (c.emoji = e)}>{e}</button>
      {/each}
    </div>
  </div>
  {#if !isEstimate(c)}
    <label class="row"><span>Every year</span>
      <input type="checkbox" class="switch" checked={c.repeat === 'yearly'} onchange={(e) => (c.repeat = e.currentTarget.checked ? 'yearly' : 'none')} />
    </label>
  {/if}
  <label class="row"><span>Remind me a day before</span>
    <input type="checkbox" class="switch" bind:checked={c.remindDayBefore} />
  </label>
  <button class="save" type="submit" disabled={!valid}>Save</button>
  <p class="dim small when">{valid ? parseYmd(c.date).toLocaleDateString(undefined, { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' }) : 'Give it a name and a date'}</p>
</form>

<style>
  .head { display: flex; align-items: center; gap: 6px; margin-bottom: 8px; }
  h3 { flex: 1; margin: 0; font-size: 15px; font-weight: 600; }
  h4 { margin: 4px 2px 6px; font-size: 12px; font-weight: 500; }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 12px; }
  .chip { display: inline-flex; align-items: center; gap: 5px; height: 28px; padding: 0 10px; border-radius: var(--r-chip); background: var(--pill); font-size: 12px; transition: background 0.15s; }
  .chip:hover { background: var(--pill-hover); }
  .chip.sel { background: var(--accent); color: var(--on-accent); }
  form { display: flex; flex-direction: column; gap: 10px; }
  .field { display: flex; flex-direction: column; gap: 4px; font-size: 12px; min-width: 0; }
  .field input { height: 40px; padding: 0 10px; border-radius: var(--r-inner); background: var(--bg-2); border: 1px solid var(--border); color: var(--fg); font: inherit; font-size: 13px; }
  .field input:focus-visible { outline: 2px solid var(--accent); outline-offset: -1px; }
  .two { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .emoji { display: flex; flex-wrap: wrap; gap: 4px; }
  .emoji button { display: grid; place-items: center; width: 32px; height: 32px; border-radius: var(--r-inner); font-size: 16px; transition: background 0.15s, transform 0.2s var(--spring); }
  .emoji button:hover { background: var(--pill-hover); }
  .emoji button.sel { background: var(--bg-2); box-shadow: inset 0 0 0 1.5px var(--accent); }
  .row { display: flex; align-items: center; justify-content: space-between; min-height: 40px; padding: 0 2px; font-size: 13px; }
  .save { height: 36px; border-radius: var(--r-inner); background: var(--accent); color: var(--on-accent); font-weight: 600; font-size: 13px; transition: transform 0.15s var(--ease), opacity 0.15s; }
  .save:active:not(:disabled) { transform: scale(0.98); }
  .save:disabled { opacity: 0.4; }
  .save:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .small { font-size: 11.5px; margin: 0 2px; }
  .when { text-align: center; }
</style>
