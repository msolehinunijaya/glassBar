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
    got = null;
    if (!key || !m) return;
    ipc.lyricsGet(m.title, m.artist, m.album, m.durationMs, use)
      .then((l) => { if (want === key) got = l; })
      .catch(() => {});
  });

  return {
    get v() { return got; },
    get lines() { return got?.lines ?? []; },
  };
}
