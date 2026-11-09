<script lang="ts">
  // Focus music (Pomodoro settings): the YouTube links play as sound while a focus session
  // runs. Lives in the dock window, which is always there (a panel can close). The player
  // appears when a focus session starts, pauses and resumes with it, and goes when the
  // session ends, is reset or a break begins. The links play in order and start over after
  // the last; one that YouTube won't play (removed, private, an ended live stream, embedding
  // off) is skipped.
  import { nextIndex, playerUrl, youtubeId } from './music';

  let { links, running, focus, enabled }: { links: string[]; running: boolean; focus: boolean; enabled: boolean } = $props();

  const ids = $derived(links.map(youtubeId).filter((x): x is string => !!x));
  let started = $state(false);
  $effect(() => {
    if (!enabled || !focus || !ids.length) started = false;
    else if (running) started = true;
  });
  // The links as they were when it started: editing them mid-session doesn't restart it.
  let list: string[] = [];
  let src = $state('');
  $effect(() => {
    if (started && !src) { list = [...ids]; src = playerUrl(list[0], location.origin); }
    else if (!started) src = '';
  });

  let frame = $state<HTMLIFrameElement>();
  const post = (msg: object) => frame?.contentWindow?.postMessage(JSON.stringify(msg), 'https://www.youtube.com');
  const send = (func: string, args: unknown[] = []) => post({ event: 'command', func, args });
  $effect(() => { if (frame) send(running ? 'playVideo' : 'pauseVideo'); });

  // Which link plays, and how many failed in a row (after all of them, it stops trying).
  // Only loadVideoById gets a player past a link that failed: nextVideo is ignored then.
  let at = 0;
  let failed = 0;
  const next = () => { at = nextIndex(at, list.length); send('loadVideoById', [list[at]]); };
  // The player sends its events only once told someone is listening, and only hears that
  // once it's ready: say it every 250 ms until it answers (as YouTube's own iframe API does).
  let heard = false;
  let hello: ReturnType<typeof setInterval> | undefined;
  $effect(() => {
    if (!src) return;
    at = 0; failed = 0; heard = false;
    const onMsg = (e: MessageEvent) => {
      if (e.origin !== 'https://www.youtube.com' || e.source !== frame?.contentWindow) return;
      let d: { event?: string; info?: unknown };
      try { d = JSON.parse(String(e.data)); } catch { return; }
      if (!heard) { heard = true; clearInterval(hello); }
      if (d.event === 'onError') { if (++failed < list.length) next(); }
      else if (d.event === 'onStateChange' && d.info === 0) next(); // ended: the next one
      else if (d.event === 'onStateChange' && d.info === 1) failed = 0; // playing
    };
    addEventListener('message', onMsg);
    return () => { removeEventListener('message', onMsg); clearInterval(hello); };
  });
  const loaded = () => {
    let tries = 0;
    clearInterval(hello);
    hello = setInterval(() => {
      if (heard || ++tries > 60) { clearInterval(hello); return; }
      post({ event: 'listening', id: 'focus-music', channel: 'widget' });
    }, 250);
    send(running ? 'playVideo' : 'pauseVideo');
  };
</script>

{#if src}
  <iframe bind:this={frame} class="focus-music" {src} title="Focus music" allow="autoplay; encrypted-media" onload={loaded}></iframe>
{/if}

<style>
  /* Sound only: the player is kept out of sight (not display:none, which would stop it). */
  .focus-music { position: fixed; left: -4000px; top: 0; width: 320px; height: 180px; border: 0; opacity: 0; pointer-events: none; }
</style>
