<script lang="ts">
  import { CloudOff } from '@lucide/svelte';
  import type { PillProps } from '../types';
  import { describe } from './codes';
  import { deg, liveWeather } from './weather.svelte';
  import WxIcon from './WxIcon.svelte';

  let { size }: PillProps = $props();
  const wx = liveWeather();
</script>

<div class="pill" title={wx.w ? `${describe(wx.w.current.code, wx.w.current.isDay).label} · ${wx.w.place?.name ?? ''}` : 'Weather'}>
  {#if wx.w}
    <WxIcon code={wx.w.current.code} isDay={wx.w.current.isDay} />
    <span class="t">{deg(wx.w.current.temp)}</span>
    {#if size !== 'small'}<span class="dim">{describe(wx.w.current.code, wx.w.current.isDay).label}</span>{/if}
  {:else}
    <CloudOff size={16} class="dim" />
  {/if}
</div>

<style>
  .t { font-weight: 600; }
</style>
