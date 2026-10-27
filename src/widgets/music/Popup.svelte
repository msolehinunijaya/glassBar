<script lang="ts">
  import { Music, Pause, Play, Settings2, SkipBack, SkipForward, Volume2, VolumeX } from '@lucide/svelte';
  import Visualizer from './Visualizer.svelte';
  import { events, type AudioState } from '../../lib/ipc';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import { CUSTOM_DEFAULTS, LYRICS_DEFAULTS, type MusicSettings } from './settings';
  import { ipc } from '../../lib/ipc';
  import type { PopupProps } from '../types';
  import { fmt, liveMedia, positionAt, toggle } from './media.svelte';
  import Rhythm from './Rhythm.svelte';

  let { instance, settings }: PopupProps<MusicSettings> = $props();
  let showOptions = $state(false);
  // settingsOf() merges one level deep, so `custom` arrives whole or not at all: merge it
  // over the defaults here, or saving one switch would wipe every part the user never touched.
  const custom = $derived({ ...CUSTOM_DEFAULTS, ...(settings.custom ?? {}) });
  const saveCustom = (p: Record<string, unknown>) => updateWidgetSettings(instance, { custom: { ...custom, ...p } });
  const lyr = $derived({ ...LYRICS_DEFAULTS, ...(settings.lyrics ?? {}) });
  const saveLyr = (p: Record<string, unknown>) => updateWidgetSettings(instance, { lyrics: { ...lyr, ...p } });
  const openLyricsFolder = async () => { try { ipc.downloadsOpen(await ipc.lyricsFolder()); } catch { /* nothing to open */ } };
  const media = liveMedia();
  let now = $state(Date.now());
  $effect(() => { const t = setInterval(() => (now = Date.now()), 500); return () => clearInterval(t); });
  const pos = $derived(media.m ? positionAt(media.m, now) : 0);
  const frac = $derived(media.m?.durationMs ? pos / media.m.durationMs : 0);

  let audio = $state<AudioState | null>(null);
  $effect(() => {
    ipc.audioState().then((a) => (audio = a)).catch(() => {});
    const off = events.audioChanged((a) => (audio = a));
    return () => { off.then((f) => f()); };
  });
  function setVol(v: number) { if (audio) audio = { ...audio, volume: v, muted: false }; ipc.audioSetVolume(v); }

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
      { type: 'choice', key: 'style', label: 'Design', choices: [{ value: 'player', label: 'Player' }, { value: 'classic', label: 'Classic' }, { value: 'mini', label: 'Mini (Play/Pause + Next)' }, { value: 'custom', label: 'Custom (choose the parts)' }] },
      { type: 'toggle', key: 'showViz', label: 'Sound visualizer (Player design)', hint: 'Live level from your speakers' },
      { type: 'toggle', key: 'showProgress', label: 'Show progress background', hint: 'Fill grows as the song plays' },
    ]} />
    {#if settings.style !== 'custom'}
      <label class="vlen">Visualizer length <input type="range" min="32" max="200" step="4" value={settings.vizWidth}
        oninput={(e) => updateWidgetSettings(instance, { vizWidth: +e.currentTarget.value })} /> <span class="dim">{settings.vizWidth}px</span></label>
    {:else}
      <!-- The custom design: the user picks every part. Saved under settings.custom, so the
           Player / Classic / Mini designs keep their own switches untouched. -->
      <OptionsPopover title="Custom design" values={custom} save={saveCustom} options={[
        { type: 'checks', label: 'Show', min: 1, items: [
          { key: 'art', label: 'Album art' }, { key: 'title', label: 'Title' }, { key: 'artist', label: 'Artist' },
          { key: 'album', label: 'Album' }, { key: 'appName', label: 'App' }, { key: 'progress', label: 'Progress bar' }] },
        { type: 'choice', key: 'artShape', label: 'Art shape', choices: [{ value: 'circle', label: 'Circle' }, { value: 'rounded', label: 'Rounded' }, { value: 'square', label: 'Square' }] },
        { type: 'range', key: 'artSize', label: 'Art size', min: 18, max: 40, unit: 'px' },
        { type: 'checks', label: 'Buttons', items: [
          { key: 'prev', label: 'Previous' }, { key: 'playPause', label: 'Play/Pause' },
          { key: 'next', label: 'Next' }, { key: 'volume', label: 'Volume' }] },
        { type: 'checks', label: 'Time', items: [
          { key: 'elapsed', label: 'Elapsed' }, { key: 'remaining', label: 'Left' }, { key: 'duration', label: 'Total' }] },
        { type: 'choice', key: 'viz', label: 'Visualizer', choices: [
          { value: 'none', label: 'None' },
          { value: 'level', label: 'Level', hint: 'How loud it is right now. Very cheap.' },
          { value: 'spectrum', label: 'Equalizer', hint: 'The real frequency spectrum, like the Equalizer widget. Costs more; pauses on battery.' }] },
        { type: 'choice', key: 'eqLook', label: 'Equalizer look', menu: true, choices: [
          { value: 'bars', label: 'Bars' }, { value: 'mirror', label: 'Mirror' }, { value: 'glow', label: 'Glow spikes' },
          { value: 'led', label: 'LED' }, { value: 'particles', label: 'Particle wave' }, { value: 'line', label: 'Line' },
          { value: 'radial', label: 'Radial' }] },
        { type: 'range', key: 'vizWidth', label: 'Visualizer length', min: 32, max: 200, step: 4, unit: 'px' },
      ]} />
      <p class="chint dim">On a left or right dock only the art and the buttons fit, so the text, the progress bar and the visualizer are left out there.</p>
    {/if}

    {#if settings.style === 'player' || settings.style === 'custom'}
      <OptionsPopover title="Lyrics" values={lyr} save={saveLyr} options={[
        { type: 'toggle', key: 'on', label: 'Show the lyric line', hint: 'The line that is playing, beside the pill' },
        { type: 'choice', key: 'side', label: 'Side', choices: [{ value: 'left', label: 'Left' }, { value: 'right', label: 'Right' }] },
        { type: 'toggle', key: 'lookup', label: 'Look up lyrics online', hint: 'Sends the title, artist, album and length to lrclib.net' },
        { type: 'range', key: 'maxWidth', label: 'Length', min: 100, max: 420, step: 10, unit: 'px' },
        { type: 'choice', key: 'size', label: 'Text size', choices: [{ value: '11', label: '11' }, { value: '12', label: '12' }, { value: '14', label: '14' }, { value: '16', label: '16' }] },
        { type: 'choice', key: 'weight', label: 'Weight', choices: [{ value: '400', label: 'Regular' }, { value: '500', label: 'Medium' }, { value: '600', label: 'Semibold' }] },
        { type: 'checks', label: 'Style', items: [{ key: 'italic', label: 'Italic' }, { key: 'caps', label: 'CAPITALS' }] },
        { type: 'choice', key: 'anim', label: 'Each line', menu: true, choices: [
          { value: 'fade', label: 'Fades in' },
          { value: 'slide-up', label: 'Slides up' },
          { value: 'slide-side', label: 'Slides in from the side' },
          { value: 'typewriter', label: 'Types itself out' },
          { value: 'karaoke', label: 'Karaoke (fills as it is sung)' },
          { value: 'none', label: 'Just appears' }] },
      ]} />
      <p class="chint dim">
        Only lyrics with timings are used, so the line can follow the song. A <code>.lrc</code> file in GlassBar's
        lyrics folder always wins, and is the only source with the lookup off.
        <button class="btn lyrbtn" onclick={openLyricsFolder}>Open lyrics folder</button>
        Name it <code>Artist - Title.lrc</code>. Not every song has lyrics; the line stays empty then.
      </p>
    {/if}
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
  <div class="vizrow"><Visualizer active={media.m.playing} width={272} height={34} /></div>
  <div class="controls">
    <button class="icon-btn" disabled={!media.m.canPrev} onclick={() => ipc.mediaControl('prev')} aria-label="Previous"><SkipBack size={22} /></button>
    <button class="icon-btn play" onclick={() => toggle(media)} aria-label={media.m.playing ? 'Pause' : 'Play'}>
      {#if media.m.playing}<Pause size={26} />{:else}<Play size={26} />{/if}
    </button>
    <button class="icon-btn" disabled={!media.m.canNext} onclick={() => ipc.mediaControl('next')} aria-label="Next"><SkipForward size={22} /></button>
  </div>
  {#if audio}
    <div class="vol">
      <button class="icon-btn" onclick={() => audio && ipc.audioSetMute(!audio.muted)} aria-label={audio.muted ? 'Unmute' : 'Mute'}>
        {#if audio.muted}<VolumeX size={16} />{:else}<Volume2 size={16} />{/if}</button>
      <input class="slider" type="range" min="0" max="1" step="0.01" value={audio.muted ? 0 : audio.volume} style:--fill="{(audio.muted ? 0 : audio.volume) * 100}%"
             oninput={(e) => setVol(+e.currentTarget.value)} aria-label="Volume" />
      <span class="dim pct">{Math.round((audio.muted ? 0 : audio.volume) * 100)}</span>
    </div>
  {/if}
{:else}
  <div class="empty dim"><Music size={36} /><p>Nothing is playing.<br />Start music in Spotify, a browser, or any media app.</p></div>
{/if}

<style>
  .gear { position: absolute; top: 10px; right: 10px; z-index: 2; }
  .gear.on { color: var(--accent); }
  .optwrap { margin: 28px 0 8px; }
  .art { width: 180px; height: 180px; margin: 4px auto 14px; border-radius: calc(18px * var(--rs)); overflow: hidden; display: grid; place-items: center; background: var(--pill); box-shadow: var(--shadow); }
  .art img { width: 100%; height: 100%; object-fit: cover; }
  .meta { display: flex; flex-direction: column; text-align: center; gap: 2px; }
  .meta b { font-size: 15px; font-weight: 600; }
  .app { font-size: 11px; }
  .bar { height: 6px; border-radius: calc(3px * var(--rs)); background: var(--track); margin: 14px 0 4px; cursor: pointer; overflow: hidden; }
  .fill { height: 100%; background: var(--fg); border-radius: calc(3px * var(--rs)); transition: width 0.5s linear; }
  .times { display: flex; justify-content: space-between; font-size: 11px; font-variant-numeric: tabular-nums; }
  .controls { display: flex; justify-content: center; align-items: center; gap: 18px; margin-top: 8px; }
  .play { width: 52px; height: 52px; }
  .icon-btn:disabled { opacity: 0.35; }
  .empty { text-align: center; padding: 60px 10px; }
  .chint { font-size: 11px; line-height: 1.4; margin: 6px 2px 0; }
  .chint code { font-size: 10.5px; padding: 0 3px; border-radius: calc(4px * var(--rs)); background: var(--pill); }
  /* Secondary, per DESIGN_RULES section 6: ghost, with a visible hover and focus ring.
     There is no global .btn -- each panel styles its own. */
  .lyrbtn {
    display: block; margin: 7px 0; padding: 6px 12px; font-size: 12px; cursor: pointer;
    border-radius: var(--r-inner); border: 1px solid var(--line); background: var(--pill); color: var(--fg);
    transition: background 0.2s var(--ease);
  }
  .lyrbtn:hover { background: var(--pill-hover); }
  .lyrbtn:active { transform: scale(0.97); }
  .lyrbtn:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .vlen { display: flex; align-items: center; gap: 8px; font-size: 12px; margin-top: 6px; }
  .vlen input { flex: 1; }
  .vizrow { display: flex; justify-content: center; margin: 10px 0 2px; }
  .vol { display: flex; align-items: center; gap: 8px; margin-top: 10px; }
  .vol .slider { flex: 1; }
  .pct { width: 24px; text-align: right; font-size: 11px; font-variant-numeric: tabular-nums; }
</style>
