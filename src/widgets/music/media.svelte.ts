import { events, ipc, type MediaInfo, type MediaSource } from '../../lib/ipc';

export function liveMedia() {
  const s = $state<{ m: MediaInfo | null }>({ m: null });
  $effect(() => {
    ipc.mediaInfo().then((m) => { s.m = m; }).catch(() => {});
    const off = events.mediaChanged((m) => { s.m = m; });
    return () => { off.then((f) => f()); };
  });
  return s;
}

/** Current position: the player reports it rarely, so extrapolate while playing. */
export function positionAt(m: MediaInfo, now = Date.now()) {
  // Never before the reported position: a clock read a moment before the report came in
  // (the panel ticks every 0.5 s) used to put a just-clicked lyric line's start behind us.
  const p = m.playing && m.updatedAtMs > 0 ? m.positionMs + Math.max(0, now - m.updatedAtMs) : m.positionMs;
  return Math.max(0, Math.min(p, m.durationMs || p));
}

export const fmt = (ms: number) => {
  const s = Math.floor(ms / 1000);
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
};

/** Optimistic play/pause so the button feels instant. */
export function toggle(s: { m: MediaInfo | null }) {
  if (s.m) s.m = { ...s.m, playing: !s.m.playing, positionMs: positionAt(s.m), updatedAtMs: Date.now() };
  ipc.mediaControl('toggle');
}

/** The playing app, with its icon (null until it arrives). */
export type Source = MediaSource & { icon: string | null };

/** The app that is playing, asked again only when it changes. */
export function liveSource(media: { m: MediaInfo | null }) {
  let v = $state<Source | null>(null);
  let key = '';
  $effect(() => {
    const app = media.m?.app ?? '';
    if (app === key) return;
    key = app;
    if (!app) { v = null; return; }
    ipc.mediaSource().then(async (s) => {
      if (key !== app) return;
      if (!s) { v = null; return; }
      const icon = s.exe || s.aumid ? await ipc.appIcon(s.exe ?? '', s.window ?? null, s.aumid ?? null).catch(() => null) : null;
      if (key === app) v = { ...s, icon };
    }).catch(() => {});
  });
  return { get v() { return v; } };
}

/** Bring the playing app to the front (or start it). */
export const openSource = () => { ipc.mediaSourceOpen().catch(() => {}); };
