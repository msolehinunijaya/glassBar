<script lang="ts">
  import { ArrowUpDown, RefreshCw, Star } from '@lucide/svelte';
  import PanelHeader from '../../lib/PanelHeader.svelte';
  import { relTime } from '../../lib/time';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import type { PopupProps } from '../types';
  import type { CurrencySettings } from './index';
  import { convert, currencyName, fmtAmount, pairKey, parsePair } from './fx';
  import { liveRates } from './rates.svelte';

  let { instance, settings }: PopupProps<CurrencySettings> = $props();
  let amount = $state(1);
  $effect.pre(() => { amount = settings.amount; });
  const fx = liveRates(() => settings.from);
  const codes = $derived(fx.r ? Object.keys(fx.r.rates).sort() : [settings.from, settings.to]);
  const result = $derived(fx.r ? convert(amount, fx.r.rates, settings.to) : null);
  const key = $derived(pairKey(settings.from, settings.to));
  const fav = $derived(settings.favorites.includes(key));
  const set = (p: Partial<CurrencySettings>) => updateWidgetSettings(instance, p);
  const when = (ms: number) => new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(ms);
  function toggleFav() {
    set({ favorites: fav ? settings.favorites.filter((k) => k !== key) : [...settings.favorites, key] });
  }
</script>

<PanelHeader title="Currency">
  <button class="icon-btn" class:spin={fx.loading} onclick={() => fx.refresh()} aria-label="Refresh rates" title="Refresh rates"><RefreshCw size={15} /></button>
</PanelHeader>

<div class="card box">
  <div class="line">
    <input class="amt" type="number" min="0" step="any" bind:value={amount} onchange={() => set({ amount: +amount || 0 })} aria-label="Amount" />
    <select value={settings.from} onchange={(e) => set({ from: e.currentTarget.value })} aria-label="From">
      {#each codes as c (c)}<option value={c}>{c}</option>{/each}
    </select>
  </div>
  <button class="swap" onclick={() => set({ from: settings.to, to: settings.from })} aria-label="Swap"><ArrowUpDown size={15} /></button>
  <div class="line">
    <b class="res">{result == null ? '–' : fmtAmount(result)}</b>
    <select value={settings.to} onchange={(e) => set({ to: e.currentTarget.value })} aria-label="To">
      {#each codes as c (c)}<option value={c}>{c}</option>{/each}
    </select>
  </div>
  <div class="dim small names">{currencyName(settings.from)} → {currencyName(settings.to)}</div>
</div>

<div class="favs">
  <button class="fav" class:on={fav} onclick={toggleFav}><Star size={13} /> {fav ? 'Saved' : 'Save pair'}</button>
  {#each settings.favorites as k (k)}
    {@const p = parsePair(k)}
    {#if p}<button class="chip" class:sel={k === key} onclick={() => set({ from: p[0], to: p[1] })}>{p[0]} → {p[1]}</button>{/if}
  {/each}
</div>

{#if fx.r}
  <p class="small meta" class:offline={fx.r.cached} title="Rates from {when(fx.r.rateTime)}. Indicative mid-market rate, not a bank or card rate.">
    {fx.r.cached ? 'Offline' : 'Mid-market rate'} · {fx.r.source} · updated {relTime(fx.r.rateTime, Date.now())}
  </p>
{:else if fx.error}
  <p class="err small">Couldn't load rates: {fx.error}</p>
{/if}

<style>
  .meta { color: var(--fg-3); }
  .meta.offline { color: #ff9f0a; }
  .box { padding: 12px; display: flex; flex-direction: column; gap: 6px; position: relative; }
  .line { display: flex; gap: 8px; align-items: center; }
  .amt { flex: 1; min-width: 0; font-size: 20px; font-weight: 600; background: transparent; border: none; color: inherit; font-variant-numeric: tabular-nums; }
  .res { flex: 1; font-size: 22px; font-variant-numeric: tabular-nums; }
  select { padding: 6px 8px; border-radius: calc(8px * var(--rs)); background: var(--pill); color: inherit; border: none; font-size: 13px; }
  .swap { align-self: center; display: grid; place-items: center; width: 28px; height: 28px; border-radius: calc(14px * var(--rs)); background: var(--pill); }
  .swap:hover { background: var(--pill-hover); }
  .names { margin-top: 2px; }
  .favs { display: flex; flex-wrap: wrap; gap: 6px; margin: 10px 0; }
  .fav, .chip { display: inline-flex; align-items: center; gap: 4px; padding: 5px 10px; border-radius: calc(12px * var(--rs)); background: var(--pill); font-size: 11px; }
  .fav.on { color: #ffcc00; }
  .chip.sel { background: var(--accent); color: var(--on-accent); }
  .small { font-size: 11px; }
  .meta { line-height: 1.5; margin: 0; }
  .err { color: #ff453a; }
  .spin :global(svg) { animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
