<script lang="ts">
  import { ipc } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import { liveAudio, pct, volumeIcon } from './audio.svelte';

  let { size, settings }: PillProps<{ step: number }> = $props();
  const audio = liveAudio();
  const Icon = $derived(volumeIcon(audio.v));
  let el: HTMLDivElement;

  // Wheel over the pill changes volume instead of scrolling the bar.
  $effect(() => {
    const onWheel = (e: WheelEvent) => {
      if (!audio.v) return;
      e.preventDefault();
      e.stopPropagation();
      const volume = Math.min(1, Math.max(0, audio.v.volume - Math.sign(e.deltaY) * settings.step / 100));
      const wasMuted = audio.v.muted;
      audio.v = { ...audio.v, volume, muted: false };
      ipc.audioSetVolume(volume);
      if (wasMuted) ipc.audioSetMute(false);
    };
    el.addEventListener('wheel', onWheel, { passive: false });
    return () => el.removeEventListener('wheel', onWheel);
  });
</script>

<div class="pill" bind:this={el} title={audio.v ? `Volume ${pct(audio.v.volume)}` : 'Sound'}>
  <Icon size={17} />
  {#if size !== 'small' && audio.v}<span class="num">{pct(audio.v.volume)}</span>{/if}
</div>

<style>
  .num { font-variant-numeric: tabular-nums; min-width: 3ch; }
</style>
