<script lang="ts">
  import type { PillProps } from '../types';
  import type { CurrencySettings } from './index';
  import { convert, fmtAmount } from './fx';
  import { liveRates } from './rates.svelte';

  let { size, settings }: PillProps<CurrencySettings> = $props();
  const fx = liveRates(() => settings.from);
  const rate = $derived(fx.r ? convert(1, fx.r.rates, settings.to) : null);
</script>

<div class="pill" title={fx.r ? `1 ${settings.from} = ${rate == null ? '?' : fmtAmount(rate)} ${settings.to} · ${fx.r.source}${fx.r.cached ? ' (cached)' : ''}` : fx.error || 'Loading rates…'}>
  {#if size === 'small'}
    <span class="dim">{settings.to}</span><b>{rate == null ? '–' : fmtAmount(rate)}</b>
  {:else}
    <span class="dim">1 {settings.from} =</span><b>{rate == null ? '–' : fmtAmount(rate)}</b><span class="dim">{settings.to}</span>
  {/if}
  {#if fx.r?.cached}<span class="dot" title="Offline: cached rate"></span>{/if}
</div>

<style>
  b { font-variant-numeric: tabular-nums; font-size: 12px; }
  .dim { font-size: 11px; }
  .dot { width: 6px; height: 6px; border-radius: calc(3px * var(--rs)); background: #ff9f0a; }
</style>
