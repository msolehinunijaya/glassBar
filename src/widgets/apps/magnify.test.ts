import { describe, expect, it } from 'vitest';
import { bell, extraLength, magnifyScales } from './magnify';

describe('bell', () => {
  it('is 1 at the pointer and 0 at or past the reach', () => {
    expect(bell(0, 10)).toBe(1);
    expect(bell(10, 10)).toBe(0);
    expect(bell(-12, 10)).toBe(0);
  });
  it('falls smoothly and evenly on both sides', () => {
    const v = bell(5, 10);
    expect(v).toBeGreaterThan(0);
    expect(v).toBeLessThan(1);
    expect(bell(3, 10)).toBe(bell(-3, 10));
  });
});

describe('magnifyScales', () => {
  it('grows the icon under the pointer fully and its neighbours less', () => {
    const [a, b, c] = magnifyScales([0, 40, 80], 40, 40, 2);
    expect(b).toBe(2);
    expect(a).toBe(c);
    expect(a).toBeGreaterThan(1);
    expect(a).toBeLessThan(2);
  });
  it('leaves everything at normal size with no pointer', () => {
    expect(magnifyScales([0, 40, 80], null, 40, 2)).toEqual([1, 1, 1]);
  });
  it('leaves icons beyond the reach alone', () => {
    expect(magnifyScales([0, 400], 0, 40, 1.6)[1]).toBe(1);
  });
});

describe('extraLength', () => {
  it('adds up how much longer the row gets', () => {
    expect(extraLength([1, 1.5, 2], 40)).toBe(60);
  });
});
