<script lang="ts">
  // "Player" design and the "Custom" design: the same pill with different parts showing.
  // Album art (with a playing badge), title, artist, album, app name, Previous / Play-Pause /
  // Next, a thin progress line with the times, a visualizer and a volume button. Which parts
  // appear comes from layoutOf() in settings.ts -- by widget size for Player, by the user's
  // own switches for Custom.
  import { Music, Pause, Play, SkipBack, SkipForward, Volume1, Volume2, VolumeX } from '@lucide/svelte';
  import { events, ipc, type AudioState } from '../../lib/ipc';
  import { tickingNow } from '../../lib/now.svelte';
  import type { WidgetSize } from '../../lib/ipc';
  import { EQ_DEFAULTS } from '../equalizer/settings';
  import Spectrum from '../equalizer/Spectrum.svelte';
  import { subText } from './label';
  import { artRadius, layoutOf, type MusicSettings } from './settings';
  import { liveMedia, positionAt, toggle, type Source } from './media.svelte';
  import SourceBadge from './SourceBadge.svelte';
  import { wheelControl } from '../../lib/wheelIntent';
  import { perf } from '../../lib/perf.svelte';
  import Visualizer from './Visualizer.svelte';
  import { seekFrac } from './progress';
  import { clock } from '../../lib/time';

  let { size, settings, vertical, onenter, onleave, source = null }: { size: WidgetSize; settings: MusicSettings; vertical: boolean; onenter?: (e: PointerEvent) => void; onleave?: () => void; source?: Source | null } = $props();
  const media = liveMedia();
  const t = tickingNow(() => !!media.m?.playing);
  const pos = $derived(media.m ? positionAt(media.m, t.now) : 0);
  const frac = $derived(media.m?.durationMs ? Math.min(1, pos / media.m.durationMs) : 0);
  const p = $derived(layoutOf(settings, size, vertical));
  const wide = $derived(size !== 'small' && !vertical);
  const hasDur = $derived(!!media.m?.durationMs);

  // A pill-sized strip of the real spectrum: the look the user picked, the Equalizer widget's
  // own defaults for everything else, trimmed so it still reads at ~20 px tall.
  const eq = $derived({ ...EQ_DEFAULTS, style: p.eqLook, bands: 20, peaks: false, glowSize: 4, reflection: false });

  const sub = $derived(subText(p, media.m, t.now));

  // Volume: wheel over the speaker adjusts it, a click mutes. Follows Windows' own changes.
  let audio = $state<AudioState | null>(null);
  $effect(() => {
    ipc.audioState().then((a) => (audio = a)).catch(() => {});
    const off = events.audioChanged((a) => (audio = a));
    return () => { off.then((f) => f()); };
  });
  // Only a wheel that started on the speaker, and only when the option is on: a dock scroll
  // passing over it must not change the volume (see wheelIntent.ts).
  const wc = wheelControl();
  function wheelVolume(e: WheelEvent) {
    if (!audio || settings.wheelVolume === false || !wc.owns(e)) return;
    e.preventDefault(); e.stopPropagation();
    const v = Math.max(0, Math.min(1, audio.volume + (e.deltaY < 0 ? 0.02 : -0.02)));
    audio = { ...audio, volume: v, muted: false };
    ipc.audioSetVolume(v);
  }
  const VolIcon = $derived(!audio || audio.muted || audio.volume === 0 ? VolumeX : audio.volume < 0.5 ? Volume1 : Volume2);
  const stop = (f: () => void) => (e: MouseEvent) => { e.stopPropagation(); f(); };

  // The progress line is a seek bar: hover shows where a click lands, click or drag jumps
  // there (on release), arrow keys skip 5 s. Never opens the panel or drags the dock.
  let hoverAt = $state<number | null>(null); // 0–1 under the pointer
  let dragging = $state(false);
  const shown = $derived(dragging && hoverAt != null ? hoverAt : frac);
  const at = (e: PointerEvent) => { const r = (e.currentTarget as HTMLElement).getBoundingClientRect(); return seekFrac(e.clientX, r.left, r.width); };
  function seekTo(f: number) {
    if (!media.m?.durationMs) return;
    const ms = Math.round(f * media.m.durationMs);
    media.m = { ...media.m, positionMs: ms, updatedAtMs: Date.now() };
    ipc.mediaSeek(ms);
  }
  function seekDown(e: PointerEvent) {
    e.stopPropagation(); e.preventDefault();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    dragging = true; hoverAt = at(e);
  }
  function seekMove(e: PointerEvent) { hoverAt = at(e); }
  function seekUp(e: PointerEvent) {
    e.stopPropagation();
    if (dragging) seekTo(at(e));
    dragging = false;
  }
  function seekKey(e: KeyboardEvent) {
    if (!media.m?.durationMs || (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight')) return;
    e.preventDefault(); e.stopPropagation();
    seekTo(Math.min(1, Math.max(0, (pos + (e.key === 'ArrowRight' ? 5000 : -5000)) / media.m.durationMs)));
  }
  const down = (e: PointerEvent) => e.stopPropagation();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="pill player" class:wide class:vertical class:playing={media.m?.playing} onpointerenter={onenter} onpointerleave={onleave}
     aria-label={media.m ? `${media.m.title} — ${media.m.artist}${media.m.app ? ` (${media.m.app})` : ''}` : 'Nothing playing'}>
  {#if p.art}
    <span class="artwrap">
      <span class="art" style:--art-size="{p.artSize}px" style:--art-r={artRadius(p.artShape)}>
      {#if media.m?.art}<img src={media.m.art} alt="" draggable="false" />{:else}<Music size={14} />{/if}
      {#if media.m?.playing}<span class="eq"><i></i><i></i><i></i></span>{/if}
    </span>
      {#if source}<SourceBadge s={source} />{/if}
    </span>
  {/if}

  {#if p.title || sub}
    <span class="meta">
      {#if p.title}<span class="ttl ellipsis">{media.m?.title || 'Not playing'}</span>{/if}
      {#if sub}<span class="sub ellipsis">{sub}</span>{/if}
    </span>
  {/if}

  {#if p.prev || p.playPause || p.next}
    <span class="ctls" onpointerdown={down}>
      {#if p.prev}
        <button class="ctl" disabled={!media.m?.canPrev} onclick={stop(() => ipc.mediaControl('prev'))} aria-label="Previous track"><SkipBack size={13} /></button>
      {/if}
      {#if p.playPause}
        <button class="ctl main" disabled={!media.m} onclick={stop(() => media.m && toggle(media))} aria-label={media.m?.playing ? 'Pause' : 'Play'}>
          {#if media.m?.playing}<Pause size={13} fill="currentColor" />{:else}<Play size={13} fill="currentColor" />{/if}
        </button>
      {/if}
      {#if p.next}
        <button class="ctl" disabled={!media.m?.canNext} onclick={stop(() => ipc.mediaControl('next'))} aria-label="Next track"><SkipForward size={13} /></button>
      {/if}
    </span>
  {/if}

  {#if perf.mode === 'low'}<!-- Low: no visualizer -->
  {:else if p.viz === 'level'}
    <Visualizer active={!!media.m?.playing} width={p.vizWidth} height={20} />
  {:else if p.viz === 'spectrum'}
    <Spectrum settings={eq} len={p.vizWidth} height={20} />
  {/if}

  {#if p.volume}
    <button class="ctl vol" onpointerdown={down} onpointerenter={wc.enter} onwheel={wheelVolume} onclick={stop(() => audio && ipc.audioSetMute(!audio.muted))}
            title={audio ? `Volume ${Math.round(audio.volume * 100)}% · ${settings.wheelVolume === false ? '' : 'scroll to change, '}click to mute` : 'Volume'} aria-label="Volume">
      <VolIcon size={14} />
    </button>
  {/if}

  {#if p.progress && hasDur && media.m}
    <div class="seek" class:active={dragging} role="slider" tabindex="0" aria-label="Song position"
         aria-valuemin={0} aria-valuemax={Math.round(media.m.durationMs / 1000)} aria-valuenow={Math.round(pos / 1000)} aria-valuetext="{clock(pos)} of {clock(media.m.durationMs)}"
         onpointerdown={seekDown} onpointermove={seekMove} onpointerup={seekUp} onpointercancel={() => (dragging = false)}
         onpointerleave={() => { if (!dragging) hoverAt = null; }} onclick={(e) => e.stopPropagation()} onkeydown={seekKey}>
      <span class="line"><span class:live={!dragging} style:transform="scaleX({shown})"></span></span>
      <span class="knob" style:left="{shown * 100}%"></span>
      {#if hoverAt != null}<span class="tip" style:left="{hoverAt * 100}%">{clock(hoverAt * media.m.durationMs)}</span>{/if}
    </div>
  {/if}
</div>

<style>
  .player { position: relative; gap: 8px; padding: 0 8px 0 5px; overflow: hidden; }
  .artwrap { position: relative; display: grid; flex: none; }
  .art { position: relative; width: var(--art-size); height: var(--art-size); border-radius: var(--art-r); overflow: hidden; flex: none; display: grid; place-items: center;
         background: linear-gradient(135deg, color-mix(in srgb, var(--accent) 55%, #fff), var(--accent)); color: #fff;
         box-shadow: 0 2px 6px rgba(0,0,0,0.25); transition: transform 0.3s var(--spring); }
  .art img { width: 100%; height: 100%; object-fit: cover; }
  .player:hover .art { transform: scale(1.06); }
  /* small "now playing" badge on the art */
  /* bottom-left: the bottom-right corner carries the playing app's badge */
  .eq { position: absolute; left: 2px; bottom: 2px; display: flex; align-items: flex-end; gap: 1px; height: 8px; padding: 1px 2px; border-radius: calc(3px * var(--rs)); background: rgba(0,0,0,0.55); }
  /* scaleY, not height: animating height re-lays-out and repaints the whole bar every frame. */
  .eq i { width: 2px; height: 7px; background: #fff; border-radius: calc(1px * var(--rs)); transform-origin: bottom; animation: eq 0.9s ease-in-out infinite; }
  .eq i:nth-child(2) { animation-delay: -0.3s; } .eq i:nth-child(3) { animation-delay: -0.6s; }
  @keyframes eq { 0%, 100% { transform: scaleY(0.29); } 50% { transform: scaleY(1); } }
  :global(:root[data-perf='low']) .eq i { animation: none; }
  .meta { display: flex; flex-direction: column; justify-content: center; min-width: 0; width: 120px; line-height: 1.2; }
  .ttl { font-size: 12px; font-weight: 600; }
  .sub { font-size: 10.5px; color: var(--fg-dim); font-variant-numeric: tabular-nums; }
  .ctls { display: flex; align-items: center; gap: 1px; }
  .vertical .ctls { flex-direction: column; }
  .ctl { display: grid; place-items: center; width: 24px; height: 24px; border-radius: 50%; transition: background 0.2s, transform 0.25s var(--spring); }
  .ctl:hover:not(:disabled) { background: var(--pill-hover); }
  .ctl:active:not(:disabled) { transform: scale(0.85); }
  .ctl:disabled { opacity: 0.35; }
  .ctl.main { width: 28px; height: 28px; background: var(--accent); color: var(--on-accent); box-shadow: 0 2px 6px color-mix(in srgb, var(--accent) 45%, transparent); }
  .ctl.main:hover:not(:disabled) { background: var(--accent); transform: scale(1.08); }
  /* Seek bar: a 10 px tall hit strip over the 2 px line; on hover the line thickens and a knob shows. */
  .seek { position: absolute; left: 10px; right: 10px; bottom: 0; height: 10px; cursor: pointer; z-index: 2; touch-action: none; }
  .seek:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; border-radius: calc(3px * var(--rs)); }
  .line { position: absolute; left: 0; right: 0; bottom: 3px; height: 2px; border-radius: calc(1px * var(--rs)); background: var(--track); overflow: hidden; transition: height 0.12s; }
  .line span { display: block; height: 100%; background: var(--accent); transform-origin: left; }
  .line span.live { transition: transform 1s linear; }
  .seek:hover .line, .seek.active .line { height: 4px; }
  .knob { position: absolute; bottom: 1px; width: 8px; height: 8px; margin-left: -4px; border-radius: 50%; background: var(--accent); box-shadow: 0 1px 3px rgba(0,0,0,0.4); opacity: 0; transition: opacity 0.12s; pointer-events: none; }
  .seek:hover .knob, .seek.active .knob { opacity: 1; }
  .tip { position: absolute; bottom: 9px; transform: translateX(-50%); font-size: 10px; font-variant-numeric: tabular-nums; padding: 1px 4px; border-radius: calc(4px * var(--rs)); background: rgba(0,0,0,0.75); color: #fff; pointer-events: none; white-space: nowrap; }
</style>
