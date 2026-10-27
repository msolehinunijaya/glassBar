<script lang="ts">
  import { ExternalLink, MicVocal, Music, Pause, Play, Settings2, SkipBack, SkipForward, Volume2, VolumeX } from '@lucide/svelte';
  import Visualizer from './Visualizer.svelte';
  import { events, type AudioState } from '../../lib/ipc';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import { CUSTOM_DEFAULTS, LYRICS_DEFAULTS, type MusicSettings } from './settings';
  import { ipc } from '../../lib/ipc';
  import type { PopupProps } from '../types';
  import { fmt, liveMedia, liveSource, openSource, positionAt, toggle } from './media.svelte';
  import Rhythm from './Rhythm.svelte';
  import LyricsList from './LyricsList.svelte';
  import { liveLyrics } from './lyrics.svelte';

  let { instance, settings }: PopupProps<MusicSettings> = $props();
  let showOptions = $state(false);
  // settingsOf() merges one level deep, so `custom` arrives whole or not at all: merge it
  // over the defaults here, or saving one switch would wipe every part the user never touched.
  const custom = $derived({ ...CUSTOM_DEFAULTS, ...(settings.custom ?? {}) });
  const saveCustom = (p: Record<string, unknown>) => updateWidgetSettings(instance, { custom: { ...custom, ...p } });
  const lyr = $derived({ ...LYRICS_DEFAULTS, ...(settings.lyrics ?? {}) });
  const saveLyr = (p: Record<string, unknown>) => updateWidgetSettings(instance, { lyrics: { ...lyr, ...p } });
  // Both awaited: without the second one, a failure to open the folder was an unhandled
  // rejection and the button just did nothing.
  const openLyricsFolder = async () => {
    try { await ipc.downloadsOpen(await ipc.lyricsFolder()); } catch { /* nothing to open */ }
  };
  const media = liveMedia();
  const src = liveSource(media);
  let now = $state(Date.now());
  $effect(() => { const t = setInterval(() => (now = Date.now()), 500); return () => clearInterval(t); });
  const pos = $derived(media.m ? positionAt(media.m, now) : 0);
  // The whole song's lyrics beside the player (below it in a narrow panel), when the lyric
  // switch is on -- also on a side dock, where the dock itself has no room for the line.
  const lyrics = liveLyrics(media, () => lyr.on, () => lyr.lookup);
  const showList = $derived(lyr.on && lyrics.lines.length > 0);
  // Lyrics go beside the player, however the panel was opened: wide while there are lyrics
  // to show, narrow when they are off or the song has none. Nothing changes while they are
  // still loading, so the panel doesn't shrink and grow again.
  const want = $derived(!lyr.on ? 320 : lyrics.v === null ? null : showList ? 640 : 320);
  $effect(() => { if (want !== null) ipc.popupWidth(want).catch(() => {}); });
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

<!-- Lyrics on/off at a glance, right where you look (the full lyric options are behind the gear). -->
<button class="lyrtoggle" class:on={lyr.on} onclick={() => saveLyr({ on: !lyr.on })} aria-pressed={lyr.on}
        title={lyr.on ? 'Hide lyrics (on the dock and here)' : 'Show lyrics (on the dock and here)'}>
  <MicVocal size={14} /><span>Lyrics</span>
</button>
<button class="icon-btn gear" class:on={showOptions} onclick={() => (showOptions = !showOptions)} aria-label="Music widget options"><Settings2 size={16} /></button>
{#if showOptions}
  <div class="optwrap">
    <OptionsPopover title="Music widget" values={settings} save={(p) => updateWidgetSettings(instance, p)} options={[
      { type: 'choice', key: 'style', label: 'Design', choices: [{ value: 'player', label: 'Player' }, { value: 'classic', label: 'Classic' }, { value: 'mini', label: 'Mini (Play/Pause + Next)' }, { value: 'custom', label: 'Custom (choose the parts)' }] },
      { type: 'toggle', key: 'showViz', label: 'Sound visualizer (Player design)', hint: 'Live level from your speakers' },
      { type: 'toggle', key: 'showProgress', label: 'Show progress background', hint: 'Fill grows as the song plays' },
      { type: 'toggle', key: 'hoverOpen', label: 'Open this panel on hover', hint: 'Off: only a click on the widget opens it' },
      { type: 'toggle', key: 'wheelVolume', label: 'Mouse wheel changes volume', hint: 'Over the volume button. Off: scrolling there never changes the volume' },
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

      <OptionsPopover title="Lyrics" values={lyr} save={saveLyr} options={[
        { type: 'toggle', key: 'on', label: 'Show the lyric line', hint: 'Beside the music widget, and the whole song in its panel' },
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
  </div>
{/if}
{#if media.m}
  <div class="cols" class:has-lyr={showList}>
  <div class="player">
  <div class="art">
    {#if media.m.art}<img src={media.m.art} alt="Album art" />{:else}<Music size={48} />{/if}
  </div>
  <div class="meta">
    <b class="ellipsis">{media.m.title || 'Unknown title'}</b>
    <span class="dim ellipsis">{[media.m.artist, media.m.album].filter(Boolean).join(' — ')}</span>
    <!-- The app that is playing: click to bring it to the front (or start it). -->
    <button class="srcchip" onclick={openSource} title="Open {src.v?.name ?? media.m.app}">
      <Rhythm playing={media.m.playing} />
      {#if src.v?.icon}<img src={src.v.icon} alt="" width="14" height="14" draggable="false" />{/if}
      <span>{src.v?.name ?? media.m.app}</span>
      <ExternalLink size={11} />
    </button>
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
  </div>
  {#if showList}<LyricsList lines={lyrics.lines} {media} {pos} source={lyrics.v?.source ?? 'lrclib'} />
  {:else if lyr.on && lyrics.status !== 'idle'}
    <p class="lnote" class:busy={lyrics.status === 'loading'}>{lyrics.status === 'loading' ? 'Looking for lyrics…' : lyrics.status === 'offline' ? 'Lyrics not available right now — trying again shortly' : 'Lyrics not available for this song'}</p>
  {/if}
  </div>
{:else}
  <div class="empty dim"><Music size={36} /><p>Nothing is playing.<br />Start music in Spotify, a browser, or any media app.</p></div>
{/if}

<style>
  /* With lyrics: below the player in a narrow panel, beside it in a wide one (hovering opens
     it wide). Beside, the list takes the player's height and scrolls inside it. */
  .cols.has-lyr :global(.lyrics) { height: 220px; margin-top: 14px; }
  @media (min-width: 560px) {
    .cols.has-lyr { display: grid; grid-template-columns: 272px 1fr; gap: 20px; }
    .cols.has-lyr :global(.lyrics) { height: auto; margin-top: 0; contain: size; padding-top: 24px; }
  }
  .lyrtoggle {
    position: absolute; top: 12px; right: 46px; z-index: 2; display: inline-flex; align-items: center; gap: 5px;
    height: 26px; padding: 0 10px; border-radius: var(--r-chip); font-size: 11.5px; font-weight: 500;
    background: var(--pill); color: var(--fg-dim); border: 1px solid var(--line-soft);
    transition: background 0.2s var(--ease), color 0.2s var(--ease), transform 0.15s var(--ease);
  }
  .lyrtoggle:hover { background: var(--pill-hover); color: var(--fg); }
  .lyrtoggle:active { transform: scale(0.95); }
  .lyrtoggle.on { background: var(--accent); color: var(--on-accent); border-color: transparent; }
  .lyrtoggle:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .lnote { margin: 12px 0 0; font-size: 11.5px; text-align: center; color: var(--fg-dim); font-style: italic; }
  .lnote.busy { animation: breathe 1.4s ease-in-out infinite; }
  @keyframes breathe { 50% { opacity: 0.45; } }
  .gear { position: absolute; top: 10px; right: 10px; z-index: 2; }
  .gear.on { color: var(--accent); }
  .optwrap { margin: 28px 0 8px; }
  /* Room above for the Lyrics switch and the gear, so they never sit on the cover. */
  .art { width: 180px; height: 180px; margin: 34px auto 14px; border-radius: calc(18px * var(--rs)); overflow: hidden; display: grid; place-items: center; background: var(--pill); box-shadow: var(--shadow); }
  .art img { width: 100%; height: 100%; object-fit: cover; }
  .meta { display: flex; flex-direction: column; text-align: center; gap: 2px; }
  .meta b { font-size: 15px; font-weight: 600; }
  .srcchip {
    display: inline-flex; align-items: center; gap: 6px; align-self: center; margin-top: 4px; height: 26px; padding: 0 10px;
    border-radius: var(--r-chip); font-size: 11.5px; font-weight: 500; color: var(--fg-dim);
    background: var(--pill); border: 1px solid var(--line-soft); transition: background 0.2s var(--ease), color 0.2s var(--ease), transform 0.15s var(--ease);
  }
  .srcchip:hover { background: var(--pill-hover); color: var(--fg); }
  .srcchip:active { transform: scale(0.96); }
  .srcchip:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .srcchip img { width: 14px; height: 14px; object-fit: contain; }
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
