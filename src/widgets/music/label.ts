// The text the Player and Custom designs draw. Pure, so the composition rules can be checked
// without a pill: which facts appear, in what order, and what shows when nothing is playing.
import type { MediaInfo } from '../../lib/ipc';
import { fmt, positionAt } from './media.svelte';
import type { MusicLayout } from './settings';

/** "1:23 / 4:56", or with Left on "1:23 / -3:33". Empty when the player reports no duration
 *  (a live stream or a radio), so no fake 0:00 is shown. */
export function timesText(p: MusicLayout, m: MediaInfo | null, now: number): string {
  if (!m?.durationMs) return '';
  const pos = positionAt(m, now);
  const bits: string[] = [];
  if (p.elapsed) bits.push(fmt(pos));
  if (p.remaining) bits.push('-' + fmt(m.durationMs - pos));
  if (p.duration) bits.push(fmt(m.durationMs));
  return bits.join(' / ');
}

/**
 * The line under the title: whichever facts are on, joined. The user picks the facts; keeping
 * them to one line is what stops a 36 px dock growing a paragraph (DESIGN_RULES section 9).
 *
 * "Play something" belongs under the title, so it appears only when the title does -- a small
 * pill, which shows no text at all, must not sprout a line.
 */
export function subText(p: MusicLayout, m: MediaInfo | null, now: number): string {
  if (!m) return p.title ? 'Play something' : '';
  const bits: string[] = [];
  // The Player design has always fallen back to the app name when there is no artist.
  if (p.artist) bits.push(p.oneLine ? m.artist || m.app : m.artist);
  if (p.album) bits.push(m.album);
  if (p.appName) bits.push(m.app);
  const t = timesText(p, m, now);
  if (t) bits.push(t);
  return bits.filter(Boolean).join(' · ');
}
