import { events, ipc, type MediaInfo } from '../../lib/ipc';

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
  const p = m.playing && m.updatedAtMs > 0 ? m.positionMs + (now - m.updatedAtMs) : m.positionMs;
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
