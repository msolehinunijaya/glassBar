<script lang="ts">
  // Weather on the left of the clock pill (was the Weather widget): icon above the
  // temperature, on its own condition-coloured background. Clicking it opens the full
  // weather panel; the rest of the pill still opens the clock.
  import { CloudOff } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import { describe } from '../weather/codes';
  import { themeOf } from '../weather/sun';
  import { deg, liveWeather } from '../weather/weather.svelte';
  import WxIcon from '../weather/WxIcon.svelte';

  let { instance, colorful, vertical }: { instance: string; colorful: boolean; vertical: boolean } = $props();
  const wx = liveWeather();
  const desc = $derived(wx.w ? describe(wx.w.current.code, wx.w.current.isDay) : null);

  function open(e: MouseEvent) {
    e.stopPropagation();
    ipc.popupOpen(`__weather:${instance}`, (e.currentTarget as HTMLElement).getBoundingClientRect(), 340, 640);
  }
</script>

<button class="wx" class:side={vertical} class:colorful={colorful && !!wx.w} class:vertical
        style:background={colorful && wx.w && desc ? themeOf(desc.icon, wx.w.current.isDay) : undefined}
        title={wx.w && desc ? `${desc.label} · ${wx.w.place?.name ?? ''} · click for the forecast` : 'Weather'}
        onclick={open} onpointerdown={(e) => e.stopPropagation()}>
  {#if wx.w}
    <WxIcon code={wx.w.current.code} isDay={wx.w.current.isDay} size={vertical ? 18 : 15} />
    <span class="t">{deg(wx.w.current.temp)}</span>
  {:else}
    <CloudOff size={15} class="dim" />
  {/if}
</button>

<style>
  /* Left segment of the clock pill: same height and corners, its own background. */
  .wx {
    display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 1px;
    height: 100%; padding: 0 9px; border-radius: var(--radius-pill) 4px 4px var(--radius-pill);
    background: var(--pill); line-height: 1.05; transition: filter 0.15s;
  }
  .wx:hover { filter: brightness(1.1); }
  /* Left/right dock: the top tile of the clock stack. */
  .wx.vertical { height: auto; padding: 7px 4px; border-radius: 12px 12px 4px 4px; }
  .vertical .t { font-size: 13px; }
  .t { font-weight: 600; font-size: 11px; font-variant-numeric: tabular-nums; }
  .colorful { color: #fff; text-shadow: 0 1px 2px rgba(0, 0, 0, 0.25); }
</style>
