<script lang="ts">
  // 30-day bar chart of daily tokens. One series: grey bars, today in the accent colour.
  // Several: stacked, one colour per provider. Hover (or focus) shows the day and value.
  import { stack, tokens } from './model';

  let { series }: { series: { name: string; color: string; data: [string, number][] }[] } = $props();
  const s = $derived(stack(series.map((x) => x.data)));
  const single = $derived(series.length === 1);
  let hover = $state<number | null>(null);
  const day = (d: string) => new Date(`${d}T12:00:00`).toLocaleDateString(undefined, { day: 'numeric', month: 'short' });
  const h = $derived(hover == null ? null : s.days[hover]);
</script>

<div class="chart">
  <div class="read">
    {#if h}
      <b>{day(h.date)}</b>
      {#if single}<span>{tokens(h.total)} tokens</span>
      {:else}{#each series as x, j (x.name)}<span><i style:background={x.color}></i>{x.name} {tokens(h.parts[j])}</span>{/each}{/if}
    {:else}
      <span class="k">Daily tokens, last 30 days</span>
      {#if !single}{#each series as x (x.name)}<span><i style:background={x.color}></i>{x.name}</span>{/each}{/if}
    {/if}
  </div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="bars" onpointerleave={() => (hover = null)}>
    {#each s.days as d, i (d.date)}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <div class="col" class:on={hover === i} tabindex="0" role="img" aria-label="{day(d.date)}: {tokens(d.total)} tokens"
           onpointerenter={() => (hover = i)} onfocus={() => (hover = i)} onblur={() => (hover = null)}>
        {#each d.parts as p, j (j)}
          <i style:height="{(p / s.max) * 100}%"
             style:background={single ? (i === s.days.length - 1 ? 'var(--accent)' : 'var(--fg-3)') : series[j].color}></i>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .read { display: flex; align-items: center; gap: var(--s-2); flex-wrap: wrap; min-height: 18px; font-size: 11px; color: var(--fg-dim); margin-bottom: var(--s-1); }
  .read b { color: var(--fg); font-weight: 600; }
  .read span { display: inline-flex; align-items: center; gap: 4px; }
  .read i { width: 8px; height: 8px; border-radius: 2px; }
  .k { color: var(--fg-3); }
  .bars { display: flex; align-items: stretch; gap: 2px; height: 64px; }
  .col { flex: 1; display: flex; flex-direction: column-reverse; border-radius: 2px; outline: none;
         background: linear-gradient(to top, var(--track) 0 2px, transparent 2px); }
  .col i { display: block; width: 100%; flex: none; }
  .col i:first-child { border-radius: 0 0 2px 2px; }
  .col i:last-of-type { border-top-left-radius: 2px; border-top-right-radius: 2px; }
  .col:hover, .col.on { background-color: var(--bg-2); }
  .col:focus-visible { box-shadow: 0 0 0 2px var(--accent); }
</style>
