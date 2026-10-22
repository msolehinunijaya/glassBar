<script lang="ts">
  import { Music, Pause, Play, SkipBack, SkipForward } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import { liveMedia, toggle } from './media.svelte';

  let { size, settings }: PillProps<{ showControls: boolean }> = $props();
  const media = liveMedia();
  const stop = (f: () => void) => (e: MouseEvent) => { e.stopPropagation(); f(); };
</script>

<div class="pill music" title={media.m ? `${media.m.title} — ${media.m.artist}` : 'Nothing playing'}>
  {#if media.m?.art}
    <img src={media.m.art} alt="" />
  {:else}
    <span class="noart"><Music size={14} /></span>
  {/if}
  {#if size !== 'small'}
    <span class="meta">
      <span class="ellipsis t">{media.m?.title || 'Not playing'}</span>
      {#if media.m?.artist}<span class="ellipsis dim a">{media.m.artist}</span>{/if}
    </span>
  {/if}
  {#if media.m && settings.showControls}
    {#if size === 'large'}
      <button class="ctl" disabled={!media.m.canPrev} onclick={stop(() => ipc.mediaControl('prev'))} aria-label="Previous"><SkipBack size={14} /></button>
    {/if}
    <button class="ctl" onclick={stop(() => toggle(media))} aria-label={media.m.playing ? 'Pause' : 'Play'}>
      {#if media.m.playing}<Pause size={15} />{:else}<Play size={15} />{/if}
    </button>
    {#if size === 'large'}
      <button class="ctl" disabled={!media.m.canNext} onclick={stop(() => ipc.mediaControl('next'))} aria-label="Next"><SkipForward size={14} /></button>
    {/if}
  {/if}
</div>

<style>
  .music { padding: 0 6px; gap: 8px; }
  img, .noart { width: 28px; height: 28px; border-radius: 8px; object-fit: cover; flex: none; }
  .noart { display: grid; place-items: center; background: var(--pill-hover); }
  .meta { display: flex; flex-direction: column; line-height: 1.2; max-width: 150px; min-width: 0; }
  .t { font-weight: 500; font-size: 12px; }
  .a { font-size: 11px; }
  .ctl { display: grid; place-items: center; width: 26px; height: 26px; border-radius: 50%; transition: background 0.2s, transform 0.3s var(--spring); }
  .ctl:hover { background: var(--pill-hover); }
  .ctl:active { transform: scale(0.85); }
  .ctl:disabled { opacity: 0.35; }
</style>
