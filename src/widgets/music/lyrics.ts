// Picking the lyric line that belongs to a moment in the song, and how far through it the
// music is. Pure, so the timing can be checked without a dock or a network.
import type { LyricLine } from '../../lib/ipc';

/**
 * Index of the line showing at `posMs`, or -1 before the first one. Binary search, because
 * this runs on every tick and a long song has hundreds of lines.
 *
 * A line owns the time from its own stamp until the next one, so the last line stays up to
 * the end of the song.
 */
export function currentIndex(lines: LyricLine[], posMs: number): number {
  if (!lines.length || posMs < lines[0].atMs) return -1;
  let lo = 0, hi = lines.length - 1;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (lines[mid].atMs <= posMs) lo = mid; else hi = mid - 1;
  }
  return lo;
}

/** The line at `i`, or '' when there is none (before the first, or between verses). */
export const lineAt = (lines: LyricLine[], i: number): string => (i >= 0 && i < lines.length ? lines[i].text : '');

/**
 * How far through line `i` the song is, 0..1, for the karaoke wipe. A line ends where the next
 * one starts, or at the end of the song for the last line. Without a duration the last line
 * can't be measured, so it reads as full rather than stuck at zero.
 */
export function lineProgress(lines: LyricLine[], i: number, posMs: number, durationMs: number): number {
  if (i < 0 || i >= lines.length) return 0;
  const start = lines[i].atMs;
  const end = i + 1 < lines.length ? lines[i + 1].atMs : durationMs;
  if (!(end > start)) return 1;
  return Math.max(0, Math.min(1, (posMs - start) / (end - start)));
}

/**
 * Font size and wrapping for a lyric line in a `width` px box: the chosen size if it fits;
 * else one size smaller; else, where wrapping is allowed, two lines at that smaller size
 * (CSS cuts anything past the second line). `measure(px)` is the line's width at that size.
 */
export function fitLine(measure: (px: number) => number, width: number, size: number, canWrap: boolean): { size: number; wrap: boolean } {
  if (measure(size) <= width) return { size, wrap: false };
  const smaller = Math.max(10, size - 1);
  return { size: smaller, wrap: canWrap && measure(smaller) > width };
}

/** An instrumental gap: no words at `i` (or before the first line), lasting over 5 s. */
export function inGap(lines: LyricLine[], i: number, durationMs: number): boolean {
  if (!lines.length || (i >= 0 && lines[i].text.trim())) return false;
  const start = i < 0 ? 0 : lines[i].atMs;
  const end = i + 1 < lines.length ? lines[i + 1].atMs : durationMs;
  return end - start > 5000;
}
