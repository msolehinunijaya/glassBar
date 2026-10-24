<script lang="ts">
  // Dock card. Small: a ring per connected provider (outer Claude, inner ChatGPT) filled to
  // % used. Medium: logo + the most urgent number + a dot for the other provider. Large: a
  // row per provider: logo · bar · % · time to reset. Colour by how full, never by brand.
  // Nothing connected: grey outline with a "+" badge. Crossing 90 % pulses the card once.
  import { Gauge, Plus } from '@lucide/svelte';
  import { ipc, type AiuReading } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { tickingNow } from '../../lib/now.svelte';
  import Ring from '../../lib/Ring.svelte';
  import type { PillProps } from '../types';
  import type { AiUsageSettings } from './index';
  import Logo from './Logo.svelte';
  import { byUrgency, pct, tokens, tone, TONE, tooltip, until, views, type View } from './model';

  let { size, settings, vertical }: PillProps<AiUsageSettings> = $props();
  const st = live<AiuReading[]>('aiusage', ipc.aiuState, []);
  const vs = $derived(views(st.v, settings.prefer));
  const order = $derived(byUrgency(vs));
  const hero = $derived(order[0]);
  const t = tickingNow(() => vs.some((v) => v.limit?.resetsAt), 30_000);
  const ring = (v: View | undefined) => (v?.limit ? v.limit.pct / 100 : 0);
  const col = (v: View | undefined) => TONE[tone(v?.limit?.pct)];
  const claude = $derived(vs.find((v) => v.provider === 'claude'));
  const gpt = $derived(vs.find((v) => v.provider === 'chatgpt'));
  const stale = $derived(vs.length > 0 && vs.every((v) => v.stale));

  // One pulse when a limit crosses 90 % (not on load, and no constant animation).
  let pulse = $state(false);
  const seen: Record<string, number> = {};
  $effect(() => {
    for (const v of vs) {
      const p = v.limit?.pct;
      if (p == null) continue;
      if (seen[v.provider] != null && seen[v.provider] <= 90 && p > 90) { pulse = true; setTimeout(() => (pulse = false), 1700); }
      seen[v.provider] = p;
    }
  });
  const value = (v: View) => (v.loading ? '–' : v.limit ? pct(v.limit.pct) : `${tokens(v.hot)} tok`);
</script>

{#snippet rings(s: number)}
  <!-- Rings only for published limits; without any, the logo (the numbers are in the tooltip). -->
  {#if claude?.limit && gpt?.limit}
    <Ring value={ring(claude)} size={s} stroke={3} color={col(claude)}>
      <Ring value={ring(gpt)} size={s - 10} stroke={3} color={col(gpt)} />
    </Ring>
  {:else if hero.limit}
    <Ring value={ring(hero)} size={s - 2} stroke={3.5} color={col(hero)} />
  {:else}
    <Logo provider={hero.provider} size={s - 8} />
  {/if}
{/snippet}

{#if !vs.length}
  {#if vertical}
    <div class="side empty" title={tooltip(vs, t.now)}><span class="plus"><Gauge size={16} /><i><Plus size={9} strokeWidth={3} /></i></span></div>
  {:else}
    <div class="pill empty" title={tooltip(vs, t.now)}>
      <span class="plus"><Gauge size={16} /><i><Plus size={9} strokeWidth={3} /></i></span>
      {#if size !== 'small'}<span class="dim lbl">AI Usage</span>{/if}
    </div>
  {/if}
{:else if vertical}
  <div class="side" class:pulse class:stale title={tooltip(vs, t.now)}>
    {@render rings(26)}
    <span class="side-v">{hero.loading ? '–' : hero.limit ? Math.round(hero.limit.pct) : tokens(hero.hot)}{#if hero.limit}<small>%</small>{/if}</span>
  </div>
{:else}
  <div class="pill aiu {size}" class:pulse class:stale title={tooltip(vs, t.now)}>
    {#if size === 'small'}
      {@render rings(24)}
    {:else if size === 'medium'}
      <Logo provider={hero.provider} size={15} />
      <b style:color={hero.limit && tone(hero.limit.pct) !== 'accent' ? col(hero) : null}>{value(hero)}</b>
      {#if order[1]}<i class="dot" style:background={order[1].limit ? col(order[1]) : 'var(--fg-3)'} title="{order[1].name} {value(order[1])}"></i>{/if}
    {:else}
      <span class="rows">
        {#each order as v (v.provider)}
          <span class="row">
            <Logo provider={v.provider} size={11} />
            {#if v.limit}
              <span class="bar"><i style:width="{Math.min(100, v.limit.pct)}%" style:background={col(v)}></i></span>
              <b>{pct(v.limit.pct)}</b>
              <span class="k">{v.limit.resetsAt ? `resets ${until(v.limit.resetsAt, t.now)}` : v.limit.label}</span>
            {:else}
              <b>{value(v)}</b><span class="k">{v.usage?.session ? 'session' : 'today'}</span>
            {/if}
          </span>
        {/each}
      </span>
    {/if}
  </div>
{/if}

<style>
  .aiu { gap: 6px; }
  .aiu.small { padding: 0 6px; }
  b { font-size: 12px; font-weight: 600; font-variant-numeric: tabular-nums; }
  .dot { width: 6px; height: 6px; border-radius: 50%; flex: none; }
  .stale { opacity: 0.6; }
  .rows { display: flex; flex-direction: column; justify-content: center; gap: 2px; }
  .row { display: grid; grid-template-columns: 11px 40px 30px auto; align-items: center; gap: 6px; height: 13px; }
  .row:has(> b:nth-child(2)) { grid-template-columns: 11px auto auto; }
  .row b { font-size: 11px; text-align: right; }
  .row .k { font-size: 11px; color: var(--fg-3); white-space: nowrap; }
  .bar { height: 3px; border-radius: 2px; background: var(--track); overflow: hidden; }
  .bar i { display: block; height: 100%; border-radius: inherit; transition: width 0.6s var(--ease); }
  /* Not connected: a grey outline, no fill. */
  .empty { background: transparent; border: 1px dashed var(--fg-3); box-shadow: none; color: var(--fg-3); }
  .lbl { font-size: 12px; }
  .plus { position: relative; display: grid; }
  .plus i { position: absolute; right: -4px; bottom: -3px; display: grid; place-items: center; width: 11px; height: 11px; border-radius: 50%;
            background: var(--fg-dim); color: rgb(var(--tint)); }
  .side { gap: 4px; }
  .pulse { animation: aiupulse 0.8s ease-in-out 2; }
  @keyframes aiupulse { 50% { box-shadow: 0 0 0 2px #ff453a, 0 0 14px rgba(255, 69, 58, 0.55); } }
  @media (prefers-reduced-motion: reduce) { .pulse { animation: none; } }
</style>
