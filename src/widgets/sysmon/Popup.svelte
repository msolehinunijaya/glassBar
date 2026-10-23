<script lang="ts">
  import Sparkline from '../../lib/Sparkline.svelte';
  import type { PopupProps } from '../types';
  import type { SysmonSettings } from './index';
  import { liveSamples, rate } from './samples.svelte';

  let { settings }: PopupProps<SysmonSettings> = $props();
  const s = liveSamples();
  const last = $derived(s.list.at(-1));
  const netMax = $derived(Math.max(1, ...s.list.map((x) => Math.max(x.rxBps, x.txBps))));
</script>

<h3>System</h3>
<div class="grid">
  <div class="card c"><span class="dim">CPU</span><b>{Math.round(last?.cpu ?? 0)}%</b><Sparkline values={s.list.map((x) => x.cpu)} width={130} height={40} /></div>
  <div class="card c"><span class="dim">Memory</span><b>{Math.round(last?.mem ?? 0)}%</b><Sparkline values={s.list.map((x) => x.mem)} width={130} height={40} color="#bf5af2" /></div>
  {#if settings.gpu}
    <div class="card c"><span class="dim">GPU</span><b>{last?.gpu == null ? '–' : `${Math.round(last.gpu)}%`}</b><Sparkline values={s.list.map((x) => x.gpu ?? 0)} width={130} height={40} color="#30d158" /></div>
  {/if}
  {#if settings.network}
    <div class="card c"><span class="dim">Network</span><b class="small">↓ {rate(last?.rxBps ?? 0)}<br />↑ {rate(last?.txBps ?? 0)}</b>
      <Sparkline values={s.list.map((x) => x.rxBps)} max={netMax} width={130} height={40} color="#64d2ff" /></div>
  {/if}
</div>
<p class="dim note">Updates every 2 seconds.</p>

<style>
  h3 { margin: 0 0 10px; font-size: 15px; font-weight: 600; }
  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .c { display: flex; flex-direction: column; gap: 4px; padding: 10px 12px; }
  .c b { font-size: 20px; font-variant-numeric: tabular-nums; }
  .c b.small { font-size: 12px; line-height: 1.4; }
  .note { font-size: 11px; margin-top: 10px; }
</style>
