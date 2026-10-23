import { expect, it } from 'vitest';
import { points } from './sparkline';

it('maps values to SVG polyline points', () => {
  expect(points([], 10, 10, 100)).toBe('');
  expect(points([0, 100], 10, 10, 100)).toBe('0,10 10,0');
  expect(points([50], 10, 10, 100)).toBe('10,5');
  expect(points([200, -5], 10, 10, 100)).toBe('0,0 10,10'); // clamped
});
