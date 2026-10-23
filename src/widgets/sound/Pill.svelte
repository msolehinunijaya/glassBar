<script lang="ts">
  import { ipc } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import { liveAudio, pct, volumeIcon } from './audio.svelte';
  import type { SoundSettings } from './index';

  let { size, settings }: PillProps<SoundSettings> = $props();
  const audio = liveAudio();
  const Icon = $derived(volumeIcon(audio.v));
  let el: HTMLDivElement;

  function setVolume(volume: number) {
    if (!audio.v) return;
    const wasMuted = audio.v.muted;
    audio.v = { ...audio.v, volume, muted: false };
    ipc.audioSetVolume(volume);
    if (wasMuted) ipc.audioSetMute(false);
  }

  // Wheel over the pill changes volume instead of scrolling the bar.
  $effect(() => {
    const onWheel = (e: WheelEvent) => {
      if (!audio.v) return;
      e.preventDefault();
      e.stopPropagation();
      setVolume(Math.min(1, Math.max(0, audio.v.volume - Math.sign(e.deltaY) * settings.step / 100)));
    };
    el.addEventListener('wheel', onWheel, { passive: false });
    return () => el.removeEventListener('wheel', onWheel);
  });

  const stop = (e: Event) => e.stopPropagation(); // slider use must not open the popup or start edit mode
</script>

<div class="pill" bind:this={el} title={audio.v ? `Volume ${pct(audio.v.volume)}` : 'Sound'}>
  <Icon size={17} />
  {#if settings.showSlider && audio.v}
    <input class="slider mini" type="range" min="0" max="1" step="0.01" value={audio.v.muted ? 0 : audio.v.volume}
           style:--fill={pct(audio.v.muted ? 0 : audio.v.volume)} aria-label="Volume"
           oninput={(e) => setVolume(+e.currentTarget.value)} onpointerdown={stop} onclick={stop} />
  {/if}
  {#if (size !== 'small' || settings.showSlider) && audio.v}<span class="num">{pct(audio.v.volume)}</span>{/if}
</div>

<style>
  .num { font-variant-numeric: tabular-nums; min-width: 3ch; font-size: 12px; }
  .mini { width: 90px; height: 14px; }
  .mini::-webkit-slider-runnable-track { height: 6px !important; border-radius: 3px !important; }
  .mini::-webkit-slider-thumb { width: 14px !important; height: 14px !important; margin-top: -4px; }
</style>
