import { describe, expect, it } from 'vitest';
import { clock, duration, relTime } from './time';

describe('time helpers', () => {
  it('relTime buckets', () => {
    expect(relTime(1_000_000, 1_000_000 + 30_000)).toBe('just now');
    expect(relTime(0, 5 * 60_000)).toBe('5 min ago');
    expect(relTime(0, 3 * 3_600_000)).toBe('3 h ago');
  });
  it('clock pads and shows hours only when needed', () => {
    expect(clock(65_000)).toBe('01:05');
    expect(clock(3_909_000)).toBe('1:05:09');
    expect(clock(-5)).toBe('00:00');
  });
  it('duration', () => {
    expect(duration(8040)).toBe('2h 14m');
    expect(duration(840)).toBe('14m');
    expect(duration(40)).toBe('40s');
  });
});
