import { describe, expect, it } from 'vitest';
import { fmt, positionAt } from './media.svelte';

const base = { app: '', title: '', artist: '', album: '', art: null, canNext: true, canPrev: true };

describe('positionAt', () => {
  it('extrapolates while playing and clamps to the duration', () => {
    const m = { ...base, playing: true, positionMs: 10_000, durationMs: 30_000, updatedAtMs: 1_000 };
    expect(positionAt(m, 6_000)).toBe(15_000);
    expect(positionAt(m, 999_999)).toBe(30_000);
  });
  it('holds still when paused', () => {
    const m = { ...base, playing: false, positionMs: 10_000, durationMs: 30_000, updatedAtMs: 1_000 };
    expect(positionAt(m, 6_000)).toBe(10_000);
  });
});

it('fmt renders m:ss', () => {
  expect(fmt(0)).toBe('0:00');
  expect(fmt(125_400)).toBe('2:05');
});
