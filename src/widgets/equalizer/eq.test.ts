import { expect, it } from 'vitest';
import { mix, resample, rgb, SRC_BANDS, step } from './eq';

const ramp = Array.from({ length: SRC_BANDS }, (_, i) => i / (SRC_BANDS - 1));

it('maps the full range onto n bars, low to high', () => {
  const b = resample(ramp, 16, 20, 20000, 'log');
  expect(b).toHaveLength(16);
  expect(b[0]).toBeLessThan(0.1);
  expect(b[15]).toBeGreaterThan(0.95);
  for (let i = 1; i < 16; i++) expect(b[i]).toBeGreaterThanOrEqual(b[i - 1]);
});

it('takes the loudest source band inside a wide bar', () => {
  const src = Array(SRC_BANDS).fill(0);
  src[48] = 0.9; // ~630 Hz
  expect(Math.max(...resample(src, 4, 20, 20000, 'log'))).toBe(0.9);
});

it('interpolates many narrow bars instead of stepping', () => {
  const b = resample(ramp, 128, 200, 400, 'log');
  expect(new Set(b.map((v) => v.toFixed(4))).size).toBeGreaterThan(60);
});

it('tilt lifts one end and lowers the other', () => {
  const flat = Array(SRC_BANDS).fill(0.4);
  const b = resample(flat, 8, 20, 20000, 'log', 1);
  expect(b[0]).toBeCloseTo(0.2);
  expect(b[7]).toBeCloseTo(0.8);
});

it('silent input gives n zeros', () => {
  expect(resample([], 5, 20, 20000, 'linear')).toEqual([0, 0, 0, 0, 0]);
});

it('rises fast, falls at the set speed, and peak caps hold then drop', () => {
  const s = { cur: [], peak: [], hold: [] } as { cur: number[]; peak: number[]; hold: number[] };
  for (let t = 0; t < 200; t += 16) step(s, [1], 16, t, 2, 500);
  expect(s.cur[0]).toBeGreaterThan(0.95);
  step(s, [0], 100, 300, 2, 500); // 0.1 s at 2 heights/s
  expect(s.cur[0]).toBeCloseTo(s.peak[0] - 0.2, 1);
  expect(s.peak[0]).toBeGreaterThan(0.95); // still holding
  step(s, [0], 100, 2000, 2, 500);
  expect(s.peak[0]).toBeLessThan(0.95); // hold over, falling
});

it('parses and blends hex colours', () => {
  expect(rgb('#ff8000')).toEqual([255, 128, 0]);
  expect(rgb('red')).toEqual([255, 255, 255]);
  expect(mix('#000000', '#ffffff', 0.5)).toBe('rgb(128,128,128)');
});
