<script lang="ts">
  // Player (default, see PlayerPill), Classic and Mini. Classic/Mini share a song-progress background: a soft fill
  // grows left → right with the song, pulses in the last 10 s, and hides for live streams.
  import { Music, Pause, Play, SkipBack, SkipForward } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import { tickingNow } from '../../lib/now.svelte';
  import type { PillProps } from '../types';
  import type { MusicSettings } from './index';
  import { liveMedia, toggle } from './media.svelte';
  import { songProgress } from './progress';
  import Rhythm from './Rhythm.svelte';
  import PlayerPill from './PlayerPill.svelte';

  let { size, settings, vertical }: PillProps<MusicSettings> = $props();
  const media = liveMedia();
  const t = tickingNow(() => !!media.m?.playing && settings.showProgress);
  const prog = $derived(settings.showProgress ? songProgress(media.m, t.now) : null);
  const track = $derived(media.m ? `${media.m.app}|${media.m.title}|${media.m.artist}` : '');
  const stop = (f: () => void) => (e: MouseEvent) => { e.stopPropagation(); f(); };
  const down = (e: PointerEvent) => e.stopPropagation(); // buttons must not start edit mode
</script>

{#if settings.style === 'player'}
  <PlayerPill {size} {settings} {vertical} />
{:else}
<div class="pill music" class:mini={settings.style === 'mini'} class:vertical title={media.m ? `${media.m.title} — ${media.m.artist}` : 'Nothing playing'}>
  {#if prog}
    {#key track}
      <span class="progress" class:ending={prog.ending} class:paused={!media.m?.playing} style:transform="scaleX({prog.frac})"></span>
    {/key}
  {/if}

  {#if settings.style === 'mini'}
    <!-- Compact mini player: Play/Pause and Next only. Track info lives in the tooltip and the panel. -->
    <span class="ctls">
      <button class="ctl" disabled={!media.m} onpointerdown={down} onclick={stop(() => media.m && toggle(media))} aria-label={media.m?.playing ? 'Pause' : 'Play'}>
        {#if media.m?.playing}<Pause size={14} />{:else}<Play size={14} />{/if}
      </button>
      <button class="ctl" disabled={!media.m?.canNext} onpointerdown={down} onclick={stop(() => ipc.mediaControl('next'))} aria-label="Next track"><SkipForward size={13} /></button>
    </span>
  {:else}
    {#if media.m?.art}<img class="art" src={media.m.art} alt="" />{:else}<span class="art noart"><Music size={14} /></span>{/if}
    {#if media.m}<Rhythm playing={media.m.playing} />{/if}
    {#if size !== 'small'}
      <span class="meta">
        <span class="ellipsis t">{media.m?.title || 'Not playing'}</span>
        {#if media.m?.artist}<span class="ellipsis dim a">{media.m.artist}</span>{/if}
      </span>
    {/if}
    {#if media.m && settings.showControls}
      {#if size === 'large'}
        <button class="ctl" disabled={!media.m.canPrev} onpointerdown={down} onclick={stop(() => ipc.mediaControl('prev'))} aria-label="Previous"><SkipBack size={14} /></button>
      {/if}
      <button class="ctl" onpointerdown={down} onclick={stop(() => toggle(media))} aria-label={media.m.playing ? 'Pause' : 'Play'}>
        {#if media.m.playing}<Pause size={15} />{:else}<Play size={15} />{/if}
      </button>
      {#if size === 'large'}
        <button class="ctl" disabled={!media.m.canNext} onpointerdown={down} onclick={stop(() => ipc.mediaControl('next'))} aria-label="Next"><SkipForward size={14} /></button>
      {/if}
    {/if}
  {/if}
</div>
{/if}

<style>
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

  .art { width: 28px; height: 28px; border-radius: 8px; object-fit: cover; flex: none; }
  .art.round { width: 24px; height: 24px; border-radius: 50%; }
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
</style>
