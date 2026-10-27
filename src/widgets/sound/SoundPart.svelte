<script lang="ts">
  // Volume segment of the Quick Settings pill. A ring around the speaker shows the level; the
  // speaker mutes/unmutes. Hovering slides a volume slider and a › (sound panel) open. The
  // mouse wheel only changes volume when enabled in the options. Every change plays a soft
  // tick at the new volume, so you hear how loud it is.
  import { ChevronRight } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import { volumeBlip } from '../../lib/sound';
  import { wheelControl } from '../../lib/wheelIntent';
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
  // Side docks have no room for a sideways slider: the ring only there.
  const slider = $derived(layoutOf(settings) === 'horizontal' && !vertical);
  // Ring: r = 12 in a 28 px box; the arc length follows the volume.
  const R = 12, C = 2 * Math.PI * R;
  let el = $state<HTMLDivElement>();
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

  // Only when the user turned it on: wheel over the pill changes volume -- and only a wheel
  // that started here, not a dock scroll passing over it (see wheelIntent.ts).
  const wc = wheelControl();
  $effect(() => {
    const node = el;
    if (!settings.wheel || !node) return;
    const onWheel = (e: WheelEvent) => {
      if (!audio.v || !wc.owns(e)) return;
      e.preventDefault();
      e.stopPropagation();
      setVolume(Math.min(1, Math.max(0, audio.v.volume - Math.sign(e.deltaY) * settings.step / 100)));
    };
    node.addEventListener('pointerenter', wc.enter);
    node.addEventListener('wheel', onWheel, { passive: false });
    return () => { node.removeEventListener('pointerenter', wc.enter); node.removeEventListener('wheel', onWheel); };
  });

  const stop = (e: Event) => e.stopPropagation(); // slider/mute use must not open the panel or start edit mode
  const level = $derived(audio.v ? (audio.v.muted ? 0 : audio.v.volume) : 0);
</script>

{#if vertical}
<!-- Left/right dock tile: the level ring (click to mute) and the volume under it. -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="side snd" bind:this={el} onclick={openPanel} onpointerdown={stop} role="button" tabindex="-1" onkeydown={() => {}}
     title={audio.v ? `Volume ${pct(audio.v.volume)}${audio.v.muted ? ' (muted)' : ''} · click the speaker to mute` : 'Sound'}>
  <button class="mute" class:muted={audio.v?.muted} onclick={toggleMute} onpointerdown={stop} aria-label={audio.v?.muted ? 'Unmute' : 'Mute'}>
    <svg class="ring" viewBox="0 0 28 28" aria-hidden="true">
      <circle class="track" cx="14" cy="14" r={R} />
      <circle class="arc" cx="14" cy="14" r={R} stroke-dasharray={C} stroke-dashoffset={C * (1 - level)} />
    </svg>
    <Icon size={13} />
  </button>
  {#if audio.v}<span class="side-c">{audio.v.muted ? 'Muted' : pct(audio.v.volume)}</span>{/if}
</div>
{:else}
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="pill snd" bind:this={el} onpointerenter={() => (open = slider)} onpointerleave={() => (open = false)}
     onclick={openPanel} onpointerdown={stop} role="button" tabindex="-1" onkeydown={() => {}}
     title={audio.v ? `Volume ${pct(audio.v.volume)}${audio.v.muted ? ' (muted)' : ''} · click the speaker to mute` : 'Sound'}>
  <button class="mute" class:muted={audio.v?.muted} onclick={toggleMute} onpointerdown={stop} aria-label={audio.v?.muted ? 'Unmute' : 'Mute'}>
    <svg class="ring" viewBox="0 0 28 28" aria-hidden="true">
      <circle class="track" cx="14" cy="14" r={R} />
      <circle class="arc" cx="14" cy="14" r={R} stroke-dasharray={C} stroke-dashoffset={C * (1 - level)} />
    </svg>
    <Icon size={13} />
  </button>
  {#if slider && audio.v}
    <span class="reveal" class:open style:--w="{settings.sliderWidth}px">
      <input class="slider h" style:width="{settings.sliderWidth}px" type="range" min="0" max="1" step="0.01" value={level} style:--fill={pct(level)}
             aria-label="Volume" tabindex={open ? 0 : -1}
             oninput={(e) => setVolume(+e.currentTarget.value)} onchange={(e) => setVolume(+e.currentTarget.value, true)}
             onpointerdown={stop} onclick={stop} />
      <span class="more" title="Sound panel"><ChevronRight size={14} /></span>
    </span>
  {/if}
</div>

{/if}
<style>
  .snd { gap: 2px; padding: 0 5px; }
  .side.snd { padding: 4px 4px 6px; gap: 0; }
  .mute { position: relative; display: grid; place-items: center; width: 30px; height: 30px; border-radius: 50%; transition: background 0.2s, transform 0.25s var(--spring); }
  .mute:hover { background: var(--pill-hover); }
  .mute:active { transform: scale(0.85); }
  /* Volume ring: the arc grows clockwise from the top; it glows and thickens on hover. */
  .ring { position: absolute; inset: 1px; width: 28px; height: 28px; transform: rotate(-90deg); overflow: visible; }
  .ring circle { fill: none; stroke-width: 2.4; transition: stroke-dashoffset 0.35s var(--ease), stroke-width 0.25s var(--ease), stroke 0.25s; }
  .track { stroke: var(--track); }
  .arc { stroke: var(--accent); stroke-linecap: round; }
  .snd:hover .ring circle { stroke-width: 3; }
  .snd:hover .arc { filter: drop-shadow(0 0 3px color-mix(in srgb, var(--accent) 70%, transparent)); }
  .muted .arc { stroke: transparent; }
  .muted { color: var(--fg-dim); }
  .more { display: grid; place-items: center; margin-left: 4px; opacity: 0.7; }
  .more:hover { opacity: 1; }
  /* Slider slides open on hover (width 0 → its set length). */
  .reveal { display: inline-flex; align-items: center; width: 0; overflow: hidden; opacity: 0;
            transition: width 0.28s var(--ease), opacity 0.2s, margin 0.28s var(--ease); }
  .reveal.open { width: calc(var(--w) + 24px); opacity: 1; margin: 0 2px 0 4px; }
  .h { height: 14px; flex: none; }
  .h::-webkit-slider-runnable-track { height: 6px !important; border-radius: calc(3px * var(--rs)) !important; }
  .h::-webkit-slider-thumb { width: 14px !important; height: 14px !important; margin-top: -4px; }
</style>
