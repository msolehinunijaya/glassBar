<script lang="ts">
  // Volume segment of the Quick Settings pill: the speaker icon mutes/unmutes, the rest opens
  // the sound panel. "Horizontal": icon + percentage; hovering slides a volume slider open. The mouse
  // wheel only changes volume when enabled in the options. Every change plays a soft tick at
  // the new volume, so you hear how loud it is.
  import { ipc } from '../../lib/ipc';
  import { volumeBlip } from '../../lib/sound';
  import type { WidgetSize } from '../../lib/ipc';
  import { liveAudio, pct, volumeIcon } from './audio.svelte';
  import { layoutOf, SOUND_PANEL, type SoundSettings } from './settings';

  let { instance, size, settings, vertical }: { instance: string; size: WidgetSize; settings: SoundSettings; vertical: boolean } = $props();
  function openPanel(e: MouseEvent) {
    e.stopPropagation();
    ipc.popupOpen(SOUND_PANEL + instance, (e.currentTarget as HTMLElement).getBoundingClientRect(), 340, 520);
  }
  const audio = liveAudio();
  const Icon = $derived(volumeIcon(audio.v));
  // Side docks have no room for a sideways slider: icon + percentage only there.
  const slider = $derived(layoutOf(settings) === 'horizontal' && !vertical);
  const showPct = $derived(slider || size !== 'small' || vertical);
  let el: HTMLDivElement;
  let open = $state(false);

  function setVolume(volume: number, final = false) {
    if (!audio.v) return;
    const wasMuted = audio.v.muted;
    audio.v = { ...audio.v, volume, muted: false };
    ipc.audioSetVolume(volume);
    if (wasMuted) ipc.audioSetMute(false);
    volumeBlip(final);
  }
  function toggleMute(e: Event) {
    e.stopPropagation();
    if (!audio.v) return;
    audio.v = { ...audio.v, muted: !audio.v.muted };
    ipc.audioSetMute(audio.v.muted);
    if (!audio.v.muted) volumeBlip(true);
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

  const stop = (e: Event) => e.stopPropagation(); // slider/mute use must not open the panel or start edit mode
  const level = $derived(audio.v ? (audio.v.muted ? 0 : audio.v.volume) : 0);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="pill snd" class:vertical bind:this={el} onpointerenter={() => (open = slider)} onpointerleave={() => (open = false)}
     onclick={openPanel} onpointerdown={stop} role="button" tabindex="-1" onkeydown={() => {}}
     title={audio.v ? `Volume ${pct(audio.v.volume)}${audio.v.muted ? ' (muted)' : ''} · click the speaker to mute` : 'Sound'}>
  <button class="mute" onclick={toggleMute} onpointerdown={stop} aria-label={audio.v?.muted ? 'Unmute' : 'Mute'}><Icon size={16} /></button>
  {#if slider && audio.v}
    <span class="reveal" class:open style:--w="{settings.sliderWidth}px">
      <input class="slider h" style:width="{settings.sliderWidth}px" type="range" min="0" max="1" step="0.01" value={level} style:--fill={pct(level)}
             aria-label="Volume" tabindex={open ? 0 : -1}
             oninput={(e) => setVolume(+e.currentTarget.value)} onchange={(e) => setVolume(+e.currentTarget.value, true)}
             onpointerdown={stop} onclick={stop} />
    </span>
  {/if}
  {#if showPct && audio.v}<span class="num">{Math.round(level * 100)}%</span>{/if}
</div>

<style>
  .snd { gap: 2px; padding: 0 8px 0 4px; }
  .vertical { flex-direction: column; padding: 4px 0; gap: 0; }
  .num { font-variant-numeric: tabular-nums; min-width: 3.5ch; font-size: 12px; text-align: right; }
  .vertical .num { font-size: 10.5px; text-align: center; }
  .mute { display: grid; place-items: center; width: 26px; height: 26px; border-radius: 50%; transition: background 0.2s, transform 0.25s var(--spring); }
  .mute:hover { background: var(--pill-hover); }
  .mute:active { transform: scale(0.85); }
  /* Slider slides open on hover (width 0 → its set length). */
  .reveal { display: inline-flex; align-items: center; width: 0; overflow: hidden; opacity: 0;
            transition: width 0.28s var(--ease), opacity 0.2s, margin 0.28s var(--ease); }
  .reveal.open { width: calc(var(--w) + 4px); opacity: 1; margin: 0 4px; }
  .h { height: 14px; flex: none; }
  .h::-webkit-slider-runnable-track { height: 6px !important; border-radius: 3px !important; }
  .h::-webkit-slider-thumb { width: 14px !important; height: 14px !important; margin-top: -4px; }
</style>
