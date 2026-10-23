<script lang="ts">
  import Sparkline from '../../lib/Sparkline.svelte';
  import type { PillProps } from '../types';
  import type { SysmonSettings } from './index';
  import { liveSamples, rate } from './samples.svelte';

  let { size, settings }: PillProps<SysmonSettings> = $props();
  const s = liveSamples();
  const last = $derived(s.list.at(-1));
</script>

<div class="pill" title="CPU / RAM">
  <span class="m"><span class="dim k">CPU</span>{Math.round(last?.cpu ?? 0)}%</span>
  {#if size !== 'small'}<Sparkline values={s.list.map((x) => x.cpu)} width={36} height={16} />{/if}
  <span class="m"><span class="dim k">RAM</span>{Math.round(last?.mem ?? 0)}%</span>
  {#if size !== 'small'}<Sparkline values={s.list.map((x) => x.mem)} width={36} height={16} color="#bf5af2" />{/if}
  {#if size === 'large' && settings.network && last}
    <span class="m net"><span class="dim k">↓</span>{rate(last.rxBps)} <span class="dim k">↑</span>{rate(last.txBps)}</span>
  {/if}
</div>

<style>
  .m { font-variant-numeric: tabular-nums; font-size: 12px; font-weight: 500; }
  .k { font-size: 10px; margin-right: 3px; }
  .net { min-width: 130px; }
</style>
