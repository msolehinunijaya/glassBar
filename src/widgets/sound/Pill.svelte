<script lang="ts">
  // Volume on the bar. The mouse wheel does NOT change volume unless enabled in the
  // widget options (scrolling then just scrolls the dock). Layouts: icon only,
  // horizontal slider, or a compact vertical slider. With a slider shown, the speaker
  // icon toggles mute; otherwise a click opens the full sound panel.
  import { ipc } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import { liveAudio, pct, volumeIcon } from './audio.svelte';
  import { layoutOf, type SoundSettings } from './index';

  let { size, settings }: PillProps<SoundSettings> = $props();
  const audio = liveAudio();
  const Icon = $derived(volumeIcon(audio.v));
  const layout = $derived(layoutOf(settings));
  let el: HTMLDivElement;

  function setVolume(volume: number) {
    if (!audio.v) return;
    const wasMuted = audio.v.muted;
    audio.v = { ...audio.v, volume, muted: false };
    ipc.audioSetVolume(volume);
    if (wasMuted) ipc.audioSetMute(false);
  }
  function toggleMute(e: Event) {
    e.stopPropagation();
    if (!audio.v) return;
    audio.v = { ...audio.v, muted: !audio.v.muted };
    ipc.audioSetMute(audio.v.muted);
  }

  // Only when the user turned it on: wheel over the pill changes volume.
  $effect(() => {
    if (!settings.wheel) return;
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
  const level = $derived(audio.v ? (audio.v.muted ? 0 : audio.v.volume) : 0);
</script>

<div class="pill" class:vertical-layout={layout === 'vertical'} bind:this={el} title={audio.v ? `Volume ${pct(audio.v.volume)}${audio.v.muted ? ' (muted)' : ''}` : 'Sound'}>
  {#if layout !== 'icon' && audio.v}
    <button class="mute" onclick={toggleMute} onpointerdown={stop} aria-label={audio.v.muted ? 'Unmute' : 'Mute'}><Icon size={16} /></button>
    {#if layout === 'horizontal'}
      <input class="slider h" type="range" min="0" max="1" step="0.01" value={level} style:--fill={pct(level)}
             aria-label="Volume" oninput={(e) => setVolume(+e.currentTarget.value)} onpointerdown={stop} onclick={stop} />
    {:else}
      <input class="v" type="range" min="0" max="1" step="0.01" value={level} style:--fill={pct(level)}
             aria-label="Volume" oninput={(e) => setVolume(+e.currentTarget.value)} onpointerdown={stop} onclick={stop} />
    {/if}
    <span class="num">{Math.round(level * 100)}</span>
  {:else}
    <Icon size={17} />
    {#if size !== 'small' && audio.v}<span class="num">{pct(audio.v.volume)}</span>{/if}
  {/if}
</div>

<style>
  .num { font-variant-numeric: tabular-nums; min-width: 2.5ch; font-size: 12px; }
  .mute { display: grid; place-items: center; width: 24px; height: 24px; border-radius: 50%; transition: background 0.2s; }
  .mute:hover { background: var(--pill-hover); }
  .h { width: 90px; height: 14px; }
  .h::-webkit-slider-runnable-track { height: 6px !important; border-radius: 3px !important; }
  .h::-webkit-slider-thumb { width: 14px !important; height: 14px !important; margin-top: -4px; }
  /* vertical: a narrow column that fits the bar's height */
  .vertical-layout { gap: 4px; padding: 0 6px; }
  .v {
    -webkit-appearance: none; appearance: none; writing-mode: vertical-lr; direction: rtl;
    width: 8px; height: 100%; max-height: 34px; margin: 0; border-radius: 4px;
    background: linear-gradient(to top, var(--accent) var(--fill), var(--track) var(--fill));
  }
  .v::-webkit-slider-thumb { -webkit-appearance: none; width: 12px; height: 12px; border-radius: 50%; background: #fff; box-shadow: 0 1px 3px rgba(0,0,0,.35); }
</style>
