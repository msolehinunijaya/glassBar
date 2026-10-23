<script lang="ts">
  import { Music, Pause, Play, Settings2, SkipBack, SkipForward } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { MusicSettings } from './index';
  import { ipc } from '../../lib/ipc';
  import type { PopupProps } from '../types';
  import { fmt, liveMedia, positionAt, toggle } from './media.svelte';
  import Rhythm from './Rhythm.svelte';

  let { instance, settings }: PopupProps<MusicSettings> = $props();
  let showOptions = $state(false);
  const media = liveMedia();
  let now = $state(Date.now());
  $effect(() => { const t = setInterval(() => (now = Date.now()), 500); return () => clearInterval(t); });
  const pos = $derived(media.m ? positionAt(media.m, now) : 0);
  const frac = $derived(media.m?.durationMs ? pos / media.m.durationMs : 0);

  function seek(e: MouseEvent) {
    if (!media.m?.durationMs) return;
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const ms = Math.round(((e.clientX - r.left) / r.width) * media.m.durationMs);
    media.m = { ...media.m, positionMs: ms, updatedAtMs: Date.now() };
    ipc.mediaSeek(ms);
  }
</script>

<button class="icon-btn gear" class:on={showOptions} onclick={() => (showOptions = !showOptions)} aria-label="Music widget options"><Settings2 size={16} /></button>
{#if showOptions}
  <div class="optwrap">
    <OptionsPopover title="Music widget" values={settings} save={(p) => updateWidgetSettings(instance, p)} options={[
      { type: 'choice', key: 'style', label: 'Design', choices: [{ value: 'classic', label: 'Classic' }, { value: 'mini', label: 'Mini' }] },
      { type: 'toggle', key: 'showProgress', label: 'Show progress background', hint: 'Fill grows as the song plays' },
    ]} />
  </div>
{/if}
{#if media.m}
  <div class="art">
    {#if media.m.art}<img src={media.m.art} alt="Album art" />{:else}<Music size={48} />{/if}
  </div>
  <div class="meta">
    <b class="ellipsis">{media.m.title || 'Unknown title'}</b>
    <span class="dim ellipsis">{[media.m.artist, media.m.album].filter(Boolean).join(' — ')}</span>
    <span class="dim app"><Rhythm playing={media.m.playing} /> {media.m.app}</span>
  </div>
  {#if media.m.durationMs > 0}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="bar" onclick={seek}><div class="fill" style:width="{frac * 100}%"></div></div>
    <div class="times dim"><span>{fmt(pos)}</span><span>{fmt(media.m.durationMs)}</span></div>
  {/if}
  <div class="controls">
    <button class="icon-btn" disabled={!media.m.canPrev} onclick={() => ipc.mediaControl('prev')} aria-label="Previous"><SkipBack size={22} /></button>
    <button class="icon-btn play" onclick={() => toggle(media)} aria-label={media.m.playing ? 'Pause' : 'Play'}>
      {#if media.m.playing}<Pause size={26} />{:else}<Play size={26} />{/if}
    </button>
    <button class="icon-btn" disabled={!media.m.canNext} onclick={() => ipc.mediaControl('next')} aria-label="Next"><SkipForward size={22} /></button>
  </div>
{:else}
  <div class="empty dim"><Music size={36} /><p>Nothing is playing.<br />Start music in Spotify, a browser, or any media app.</p></div>
{/if}

<style>
  .gear { position: absolute; top: 10px; right: 10px; z-index: 2; }
  .gear.on { color: var(--accent); }
  .optwrap { margin: 28px 0 8px; }
  .art { width: 180px; height: 180px; margin: 4px auto 14px; border-radius: 18px; overflow: hidden; display: grid; place-items: center; background: var(--pill); box-shadow: var(--shadow); }
  .art img { width: 100%; height: 100%; object-fit: cover; }
  .meta { display: flex; flex-direction: column; text-align: center; gap: 2px; }
  .meta b { font-size: 15px; font-weight: 600; }
  .app { font-size: 11px; }
  .bar { height: 6px; border-radius: 3px; background: var(--track); margin: 14px 0 4px; cursor: pointer; overflow: hidden; }
  .fill { height: 100%; background: var(--fg); border-radius: 3px; transition: width 0.5s linear; }
  .times { display: flex; justify-content: space-between; font-size: 11px; font-variant-numeric: tabular-nums; }
  .controls { display: flex; justify-content: center; align-items: center; gap: 18px; margin-top: 8px; }
  .play { width: 52px; height: 52px; }
  .icon-btn:disabled { opacity: 0.35; }
  .empty { text-align: center; padding: 60px 10px; }
</style>
