import { ipc, type Lyrics, type MediaInfo } from '../../lib/ipc';

/**
 * Lyrics for whatever is playing, fetched once per track.
 *
 * `media:changed` fires on every timeline update, so this asks Rust only when the artist or
 * the title actually changes. Rust owns the cache and the network, and answers from a local
 * .lrc or the kv cache when `lookup` is off.
 */
export function liveLyrics(media: { m: MediaInfo | null }, on: () => boolean, lookup: () => boolean) {
  let got = $state<Lyrics | null>(null);
  let want = ''; // the track the latest request was for; a late reply for another is dropped

  let retry: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    const m = media.m;
    // The lookup switch is part of the key, so turning it on asks again straight away instead
    // of waiting for the next song.
    const use = lookup();
    // The length is in the key too: at a track change a player can report the previous song's
    // length for a moment, and Rust doesn't remember a miss caused by a length that disagrees,
    // so asking again with the real length finds them.
    const key = on() && m?.title ? `${m.artist}\u0000${m.title}\u0000${use}\u0000${Math.round((m.durationMs || 0) / 1000)}` : '';
    if (key === want) return;
    want = key;
    clearTimeout(retry);
    got = null;
    if (!key || !m) return;
    // LRCLIB couldn't be reached: try again after 5 s, 15 s, 30 s, then every minute, for as
    // long as this song plays (a new song or turning lyrics off changes `want` and stops it).
    const delays = [5_000, 15_000, 30_000];
    let tries = 0;
    const ask = () => ipc.lyricsGet(m.title, m.artist, m.album, m.durationMs, use)
      .then((l) => {
        if (want !== key) return;
        got = l;
        if (l.source === 'offline') retry = setTimeout(() => { if (want === key) ask(); }, delays[tries++] ?? 60_000);
      })
      .catch(() => {});
    ask();
  });
  $effect(() => () => clearTimeout(retry));

  return {
    get v() { return got; },
    get lines() { return got?.lines ?? []; },
    /** 'idle' nothing to look up · 'loading' asked, no answer yet · 'found' · 'none' this song
     *  has no synced lyrics · 'offline' LRCLIB couldn't be reached (asking again shortly). */
    get status(): 'idle' | 'loading' | 'found' | 'none' | 'offline' {
      if (!want) return 'idle';
      if (!got) return 'loading';
      return got.lines.length ? 'found' : got.source === 'offline' ? 'offline' : 'none';
    },
  };
}
