<script lang="ts">
  import { CloudOff } from '@lucide/svelte';
  import type { PillProps } from '../types';
  import type { WeatherSettings } from './index';
  import { themeOf } from './sun';
  import { describe } from './codes';
  import { deg, liveWeather } from './weather.svelte';
  import WxIcon from './WxIcon.svelte';

  let { size, settings }: PillProps<WeatherSettings> = $props();
  const wx = liveWeather();
</script>

<div class="pill" class:colorful={settings.colorful && wx.w}
     style:background={settings.colorful && wx.w ? themeOf(describe(wx.w.current.code, wx.w.current.isDay).icon, wx.w.current.isDay) : undefined}
     title={wx.w ? `${describe(wx.w.current.code, wx.w.current.isDay).label} · ${wx.w.place?.name ?? ''}` : 'Weather'}>
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
  .colorful { color: #fff; text-shadow: 0 1px 2px rgba(0,0,0,0.25); }
  .colorful :global(.dim) { opacity: 0.85; color: #fff; }
</style>
