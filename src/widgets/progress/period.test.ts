import { describe, expect, it } from 'vitest';
import { period } from './period';

const at = (y: number, m: number, d: number, h = 0, min = 0) => new Date(y, m - 1, d, h, min);

describe('period progress (local time)', () => {
  it('day: midnight to midnight', () => {
    expect(period(at(2026, 10, 23, 12), 'day').frac).toBeCloseTo(0.5);
    expect(period(at(2026, 10, 23, 0), 'day').frac).toBe(0);
  });
  it('month: uses the real month length', () => {
    expect(period(at(2026, 2, 15), 'month').frac).toBeCloseTo(14 / 28); // February 2026 has 28 days
    expect(period(at(2028, 2, 15), 'month').frac).toBeCloseTo(14 / 29); // leap February
    expect(period(at(2026, 10, 1), 'month').frac).toBe(0);
  });
  it('year: leap years have 366 days', () => {
    expect(period(at(2026, 7, 2, 12), 'year').frac).toBeCloseTo(182.5 / 365);
    expect(period(at(2028, 7, 2), 'year').frac).toBeCloseTo(183 / 366);
    expect(period(at(2027, 1, 1), 'year').frac).toBe(0);
  });
  it('reports remaining time to the end of the period', () => {
    const p = period(at(2026, 10, 23, 18), 'day');
    expect(p.end - p.now).toBe(6 * 3_600_000);
  });
});
