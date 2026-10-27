<script lang="ts">
  // Player (default, see PlayerPill), Classic and Mini. Classic/Mini share a song-progress background: a soft fill
  // grows left → right with the song, pulses in the last 10 s, and hides for live streams.
  // On every design the lyric line can join the player as a second part of the same widget
  // (like Quick Settings' parts): it slides open when the track has lyrics, away when not.
  import { Music, Pause, Play, SkipBack, SkipForward } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import { tickingNow } from '../../lib/now.svelte';
  import type { PillProps } from '../types';
  import { lyricsOf, showLyrics, type MusicSettings } from './settings';
  import { liveMedia, liveSource, positionAt, toggle } from './media.svelte';
  import SourceBadge from './SourceBadge.svelte';
  import { fade, slide } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { currentIndex, inGap, lineAt, lineProgress } from './lyrics';
  import { liveLyrics } from './lyrics.svelte';
  import LyricLine from './LyricLine.svelte';
  import { songProgress } from './progress';
  import Rhythm from './Rhythm.svelte';
  import PlayerPill from './PlayerPill.svelte';
  import { hoverPanel } from '../../lib/hoverPanel';

  let { instance, size, settings, vertical }: PillProps<MusicSettings> = $props();
  const media = liveMedia();
  const src = liveSource(media);
  // Lyrics: Rust answers from a local .lrc or its cache, and only reaches the network when
  // the lookup is on. The line follows the song once a second, only while it plays.
  const lyr = $derived(lyricsOf(settings));
  const lyricsOn = $derived(showLyrics(settings, vertical));
  const lyrics = liveLyrics(media, () => lyricsOn, () => lyr.lookup);
  const lt = tickingNow(() => lyricsOn && !!media.m?.playing);
  const lpos = $derived(media.m ? positionAt(media.m, lt.now) : 0);
  const lyricIdx = $derived(currentIndex(lyrics.lines, lpos));
  const lyricShown = $derived(lyricsOn && lyrics.lines.length > 0);
  // With lyrics on, the lyric part is there for every song: the line, or a quiet note that
  // this song has none (or that they can't be fetched right now) -- never just nothing.
  // Only while the music plays: paused or stopped, the lyric part slides away (and back on play).
  const lyricPart = $derived(lyricsOn && !!media.m?.title && !!media.m?.playing && lyrics.status !== 'idle');
  const NOTE = { loading: 'Looking for lyrics…', none: 'Lyrics not available', offline: 'Lyrics not available right now', found: '', idle: '' } as const;
  // Hovering the widget opens its panel (art, controls, volume) without a click; with lyrics
  // it opens wide enough to show them beside the player.
  const hp = hoverPanel();
  // Hovering opens the panel only when that option is on (off by default: a click opens it).
  const enter = (e: PointerEvent) => { if (settings.hoverOpen) hp.enter(e, `__hover:${instance}`, lyricShown ? 640 : 320, 560); };
  const t = tickingNow(() => !!media.m?.playing && settings.showProgress);
  const prog = $derived(settings.showProgress ? songProgress(media.m, t.now) : null);
  const track = $derived(media.m ? `${media.m.app}|${media.m.title}|${media.m.artist}` : '');
  const stop = (f: () => void) => (e: MouseEvent) => { e.stopPropagation(); f(); };
  const down = (e: PointerEvent) => e.stopPropagation(); // buttons must not start edit mode
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="mwrap" class:vertical class:lyr-left={lyr.side === 'left'} data-popup-width={lyricShown ? 640 : 320} onpointerenter={enter} onpointerleave={hp.leave} onpointerdown={hp.cancel}>
{#if settings.style === 'player' || settings.style === 'custom'}
  <!-- Custom is the Player pill with the parts the user chose; see settings.ts layoutOf(). -->
  <PlayerPill {size} {settings} {vertical} source={src.v} />
{:else}
<div class="pill music" class:mini={settings.style === 'mini'} class:vertical aria-label={media.m ? `${media.m.title} — ${media.m.artist}` : 'Nothing playing'}>
  {#if prog}
    {#key track}
      <span class="progress" class:ending={prog.ending} class:paused={!media.m?.playing} style:transform="scaleX({prog.frac})"></span>
    {/key}
  {/if}

  {#if settings.style === 'mini'}
    <!-- Compact mini player: album art (when the app gives one), Play/Pause and Next. Track info lives in the tooltip and the panel. -->
    {#if media.m?.art}<span class="artwrap"><img class="art mini-art" src={media.m.art} alt="" draggable="false" />{#if src.v}<SourceBadge s={src.v} size={11} />{/if}</span>{/if}
    <span class="ctls">
      <button class="ctl" disabled={!media.m} onpointerdown={down} onclick={stop(() => media.m && toggle(media))} aria-label={media.m?.playing ? 'Pause' : 'Play'}>
        {#if media.m?.playing}<Pause size={14} />{:else}<Play size={14} />{/if}
      </button>
      <button class="ctl" disabled={!media.m?.canNext} onpointerdown={down} onclick={stop(() => ipc.mediaControl('next'))} aria-label="Next track"><SkipForward size={13} /></button>
    </span>
  {:else}
    <span class="artwrap">
      {#if media.m?.art}<img class="art" src={media.m.art} alt="" />{:else}<span class="art noart"><Music size={14} /></span>{/if}
      {#if src.v}<SourceBadge s={src.v} />{/if}
    </span>
    {#if media.m}<Rhythm playing={media.m.playing} />{/if}
    {#if size !== 'small'}
      <span class="meta">
        <span class="ellipsis t">{media.m?.title || 'Not playing'}</span>
        {#if media.m?.artist}<span class="ellipsis dim a">{media.m.artist}</span>{/if}
      </span>
    {/if}
    {#if media.m && settings.showControls}
      <button class="ctl" disabled={!media.m.canPrev} onpointerdown={down} onclick={stop(() => ipc.mediaControl('prev'))} aria-label="Previous"><SkipBack size={14} /></button>
      <button class="ctl" onpointerdown={down} onclick={stop(() => toggle(media))} aria-label={media.m.playing ? 'Pause' : 'Play'}>
        {#if media.m.playing}<Pause size={15} />{:else}<Play size={15} />{/if}
      </button>
      <button class="ctl" disabled={!media.m.canNext} onpointerdown={down} onclick={stop(() => ipc.mediaControl('next'))} aria-label="Next"><SkipForward size={14} /></button>
    {/if}
  {/if}
</div>
{/if}
{#if lyricPart}
  <!-- The lyric part. A click goes on to the dock slot, which opens the panel with the lyrics. -->
  <div class="pill lyrseg" class:note={!lyricShown} transition:slide={{ axis: 'x', duration: 260, easing: cubicOut }}>
    {#if lyricShown}
      <LyricLine text={lineAt(lyrics.lines, lyricIdx)} idx={lyricIdx} progress={lineProgress(lyrics.lines, lyricIdx, lpos, media.m?.durationMs ?? 0)}
                 gap={inGap(lyrics.lines, lyricIdx, media.m?.durationMs ?? 0)} side={lyr.side} opts={lyr} />
    {:else}
      {#key lyrics.status}<span class="lnote" class:busy={lyrics.status === 'loading'} in:fade={{ duration: 200 }}>{NOTE[lyrics.status]}</span>{/key}
    {/if}
  </div>
{/if}
</div>

<style>
  /* Player and lyric parts, joined like Quick Settings' parts: only the outer corners round. */
  .mwrap { display: flex; align-items: stretch; gap: 2px; height: 100%; }
  .mwrap.vertical { flex-direction: column; height: auto; }
  .mwrap.lyr-left .lyrseg { order: -1; }
  .mwrap > :global(.pill:not(:only-child)) { border-radius: calc(4px * var(--rs)); }
  .mwrap:not(.lyr-left) > :global(.pill:first-child), .mwrap.lyr-left > :global(.lyrseg) {
    border-top-left-radius: var(--radius-pill); border-bottom-left-radius: var(--radius-pill); }
  .mwrap:not(.lyr-left) > :global(.lyrseg), .mwrap.lyr-left > :global(.pill:first-child) {
    border-top-right-radius: var(--radius-pill); border-bottom-right-radius: var(--radius-pill); }
  .mwrap > :global(.pill:only-child) { border-radius: var(--radius-pill); }
  .lyrseg { padding: 0 12px; overflow: hidden; }
  .lnote { font-size: 11.5px; color: var(--fg-dim); white-space: nowrap; font-style: italic; }
  .lnote.busy { animation: breathe 1.4s ease-in-out infinite; }
  @keyframes breathe { 50% { opacity: 0.45; } }
  @media (prefers-reduced-motion: reduce) { .lnote.busy { animation: none; } }
  .music { position: relative; overflow: hidden; padding: 0 6px; gap: 8px; }
  .music > :not(.progress) { position: relative; z-index: 1; }
  /* progress: scaleX keeps it on the compositor; 1 s linear matches the 1 s tick */
  .progress {
    position: absolute; inset: 0; z-index: 0; transform-origin: left center;
    background: color-mix(in srgb, var(--accent) 26%, transparent);
    transition: transform 1s linear, background 0.4s;
  }
  .progress.paused { transition: none; }
  .progress.ending { background: color-mix(in srgb, #ff9f0a 34%, transparent); animation: ending 1s ease-in-out infinite; }
  @keyframes ending { 50% { opacity: 0.55; } }

  .art { width: 28px; height: 28px; border-radius: calc(8px * var(--rs)); object-fit: cover; flex: none; }
  .noart { display: grid; place-items: center; background: var(--pill-hover); }
  .meta { display: flex; flex-direction: column; line-height: 1.2; max-width: 150px; min-width: 0; }
  .t { font-weight: 500; font-size: 12px; }
  .a { font-size: 11px; }
  .ctl { display: grid; place-items: center; width: 24px; height: 24px; border-radius: 50%; transition: background 0.2s, transform 0.3s var(--spring); }
  .ctl:hover { background: var(--pill-hover); }
  .ctl:active { transform: scale(0.85); }
  .ctl:disabled { opacity: 0.35; }

  /* mini: just two small buttons, minimal padding */
  .mini { gap: 0; padding: 0 2px; }
  .ctls { display: flex; gap: 0; }
  .vertical .ctls { flex-direction: column; }
  .mini .ctl { width: 22px; height: 22px; }
  .artwrap { position: relative; display: grid; flex: none; }
  .mini-art { width: 22px; height: 22px; border-radius: calc(6px * var(--rs)); margin: 0 2px; }
</style>
