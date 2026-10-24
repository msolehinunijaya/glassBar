import { expect, it } from 'vitest';
import { barHeights, pushPeak } from './viz';

it('keeps the newest n peaks, oldest dropping off the left', () => {
  let h: number[] = [];
  for (const p of [0.1, 0.2, 0.3, 0.4]) h = pushPeak(h, p, 3);
  expect(h).toEqual([0.2, 0.3, 0.4]);
});

it('maps peaks to bar heights with a perceptual curve and a visible minimum', () => {
  const b = barHeights([0, 0.25, 1], 20);
  expect(b[0]).toBe(2); // silence still shows a hairline
  expect(b[2]).toBe(20);
  expect(b[1]).toBeGreaterThan(20 * 0.25); // quiet sounds are lifted, like a VU meter
});
