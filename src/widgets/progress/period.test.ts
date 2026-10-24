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

import { grid, label } from './period';
describe('dot grid (reference design)', () => {
  it('uses 52 week-dots for the year and 30 for month/day', () => {
    expect(grid('year')).toMatchObject({ cols: 13, rows: 4, total: 52 });
    expect(grid('month')).toMatchObject({ cols: 10, rows: 3, total: 30 });
    expect(grid('day')).toMatchObject({ cols: 10, rows: 3, total: 30 });
  });
  it('fills dots like the reference: 3% -> 1, 71% -> 21, 75% of 52 -> 39', () => {
    expect(grid('month', 0.03).filled).toBe(1);
    expect(grid('day', 0.71).filled).toBe(21);
    expect(grid('year', 0.75).filled).toBe(39);
    expect(grid('day', 0).filled).toBe(0);
    expect(grid('day', 1).filled).toBe(30);
  });
  it('labels each card like the reference', () => {
    const d = new Date(2026, 9, 23, 15);
    expect(label(d, 'month', 'en-US')).toBe('Oct 2026');
    expect(label(d, 'day', 'en-US')).toBe('Oct 23');
    expect(label(d, 'year', 'en-US')).toBe('2026');
  });
});
