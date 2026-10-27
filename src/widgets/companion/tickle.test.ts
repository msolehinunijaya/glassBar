import { describe, expect, it } from 'vitest';
import { FULL, MIN_DX, NO_TICKLE, WINDOW_MS, level, onMove, reset, type Tickle } from './tickle';

/** Walk the pointer through a list of x positions, one per `step` ms. */
function walk(xs: number[], step = 60, from: Tickle = NO_TICKLE, t0 = 1_000) {
  let t = from;
  let now = t0;
  for (const x of xs) { t = onMove(t, x, now); now += step; }
  return { t, now };
}

describe('crossing the pill is not a tickle', () => {
  it('ignores one pass straight across', () => {
    const { t, now } = walk([0, 20, 40, 60, 80, 100]);
    expect(level(t, now)).toBe(0);
  });

  it('ignores one pass back the other way', () => {
    const { t, now } = walk([100, 80, 60, 40, 20, 0]);
    expect(level(t, now)).toBe(0);
  });

  it('ignores a shaky hand holding still', () => {
    const { t, now } = walk([50, 51, 50, 52, 51, 50]);
    expect(level(t, now)).toBe(0);
    expect(MIN_DX).toBeGreaterThan(2);
  });
});

describe('going back and forth is', () => {
  it('builds up with each direction change', () => {
    const a = walk([0, 30, 0]);
    expect(level(a.t, a.now)).toBeCloseTo(1 / FULL);
    const b = walk([0, 30, 0, 30]);
    expect(level(b.t, b.now)).toBeCloseTo(2 / FULL);
  });

  it('reaches full strength after enough of them', () => {
    const { t, now } = walk([0, 30, 0, 30, 0, 30, 0]);
    expect(level(t, now)).toBe(1);
  });

  it('never goes over 1, however much you keep going', () => {
    const { t, now } = walk([0, 30, 0, 30, 0, 30, 0, 30, 0, 30, 0, 30]);
    expect(level(t, now)).toBe(1);
  });

  it('fades away once you stop', () => {
    const { t, now } = walk([0, 30, 0, 30, 0, 30, 0]);
    const last = Math.max(...t.flips);
    expect(level(t, now)).toBe(1);
    // Flips age out one by one from the oldest, so it thins out before it empties.
    expect(level(t, last + WINDOW_MS - 1)).toBeLessThan(1);
    expect(level(t, last + WINDOW_MS - 1)).toBeGreaterThan(0);
    expect(level(t, last + WINDOW_MS + 1)).toBe(0);
  });

  it('does not count slow back-and-forth spread over seconds', () => {
    const { t, now } = walk([0, 30, 0, 30, 0, 30, 0], 600);
    expect(level(t, now)).toBeLessThan(1);
  });
});

describe('leaving and coming back', () => {
  it('forgets which way it was going, so re-entering is not a flip', () => {
    const { t, now } = walk([0, 30]);
    const after = onMove(reset(t), 0, now);
    expect(level(after, now)).toBe(0);
  });

  it('keeps flips already earned while they are still fresh', () => {
    const { t, now } = walk([0, 30, 0, 30]);
    const before = level(t, now);
    expect(level(reset(t), now)).toBe(before);
  });
});

describe('the first move', () => {
  it('only records where the pointer is, and guesses no direction from it', () => {
    // Guessing one here is what made a plain walk across the dock read as a tickle: the
    // direction invented from "nowhere -> somewhere" reversed on the very next move.
    const t = onMove(NO_TICKLE, 500, 1_000);
    expect(t.lastX).toBe(500);
    expect(t.dir).toBe(0);
    expect(level(t, 1_000)).toBe(0);
  });

  it('is not a tickle however far from the origin the pointer appears', () => {
    const { t, now } = walk([900, 930, 960]);
    expect(level(t, now)).toBe(0);
  });
});
