import { describe, expect, it } from 'vitest';
import { bell, magnifyScales, onScreen, spread } from './magnify';

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

describe('spread', () => {
  // R = how far the wave reaches, S = biggest size.
  it('keeps the icon under the pointer where it is', () => {
    expect(spread(0, 100, 1.6)).toBe(0);
  });
  it('pushes icons on each side away from the pointer, evenly', () => {
    expect(spread(40, 100, 1.6)).toBeGreaterThan(0);
    expect(spread(-40, 100, 1.6)).toBeCloseTo(-spread(40, 100, 1.6));
    expect(spread(60, 100, 1.6)).toBeGreaterThan(spread(30, 100, 1.6));
  });
  it('moves icons past the wave by a fixed amount', () => {
    expect(spread(150, 100, 1.6)).toBeCloseTo(0.6 * 50);
    expect(spread(400, 100, 1.6)).toBeCloseTo(0.6 * 50);
  });
});

describe('onScreen', () => {
  it('tells whether an icon is inside the visible dock', () => {
    expect(onScreen({ left: 10, right: 50 }, 800)).toBe(true);
    expect(onScreen({ left: -300, right: -260 }, 800)).toBe(false);
    expect(onScreen({ left: 790, right: 830 }, 800)).toBe(false);
  });
});
