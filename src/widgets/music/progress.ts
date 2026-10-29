import type { MediaInfo } from '../../lib/ipc';
import { positionAt } from './media.svelte';

/** How far through the song we are, and whether it is in its last 10 seconds.
 *  Null when the length is unknown (live streams), so the fill is hidden. */
export function songProgress(m: MediaInfo | null, now: number): { frac: number; ending: boolean } | null {
  if (!m || !m.durationMs) return null;
  const pos = positionAt(m, now);
  return { frac: Math.min(1, pos / m.durationMs), ending: m.durationMs - pos <= 10_000 };
}

/** Where on the song a pointer at `x` over a bar from `left`, `width` px wide points (0–1). */
export const seekFrac = (x: number, left: number, width: number) => (width > 0 ? Math.min(1, Math.max(0, (x - left) / width)) : 0);
