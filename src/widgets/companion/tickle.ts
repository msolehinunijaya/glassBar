// Being tickled, as opposed to the pointer merely crossing the pill on its way somewhere.
//
// A tickle is going back and forth, so what counts is direction changes, not raw movement --
// otherwise walking the mouse across the dock once would set it off. Pure, so the hysteresis
// can actually be checked.

export interface Tickle {
  /** Which way the pointer was last going: 1 right, -1 left, 0 unknown. */
  dir: 0 | 1 | -1;
  /** Where it was, or null when we haven't seen it yet. Null matters: a direction guessed
   *  from the gap between "nowhere" and the first real position is a direction that then
   *  reverses on the next move, which read a plain walk across the dock as a tickle. */
  lastX: number | null;
  /** When each direction change happened. */
  flips: number[];
}

export const NO_TICKLE: Tickle = { dir: 0, lastX: null, flips: [] };

/** How long a flip still counts for. Three of these inside it is a brisk wiggle, not a
 *  leisurely one -- slow back-and-forth should read as half-hearted, and does. */
export const WINDOW_MS = 900;
/** Flips within the window for a full-strength tickle. */
export const FULL = 3;
/** Movement smaller than this is jitter, or a shaky hand. */
export const MIN_DX = 4;

/** Fold one pointer move in. */
export function onMove(t: Tickle, x: number, now: number): Tickle {
  if (t.lastX === null) return { ...t, lastX: x }; // the first move only says where it is
  const dx = x - t.lastX;
  if (Math.abs(dx) < MIN_DX) return t;
  const dir = dx > 0 ? 1 : -1;
  const turned = t.dir !== 0 && dir !== t.dir;
  const flips = (turned ? [...t.flips, now] : t.flips).filter((f) => now - f <= WINDOW_MS);
  return { dir, lastX: x, flips };
}

/** How tickled it is, 0..1. Fades on its own as the flips age out of the window. */
export function level(t: Tickle, now: number): number {
  const recent = t.flips.filter((f) => now - f <= WINDOW_MS).length;
  return Math.min(1, recent / FULL);
}

/** Pointer left the pill: forget where it was and which way it was going, so coming back
 *  somewhere else isn't a reversal. The flips already earned keep fading on their own. */
export const reset = (t: Tickle): Tickle => ({ ...t, dir: 0, lastX: null });
