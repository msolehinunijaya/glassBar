import { describe, expect, it } from 'vitest';
import { duringBlip, mix, palette, resample, rgb, SRC_BANDS, step } from './eq';
import { shrunkAt } from './settings';

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

const px = (...cols: [number, number, number, number][]) => cols.flatMap(([r, g, b, n]) => Array(n).fill([r, g, b, 255]).flat());

it('album palette: strongest hue first, a far hue second', () => {
  const p = palette(px([230, 20, 20, 50], [20, 40, 230, 20], [120, 120, 120, 200]));
  expect(p).toEqual(['#e61414', '#1428e6']);
});

it('album palette: one hue gives a lighter shade, grey gives null', () => {
  expect(palette(px([0, 200, 0, 10]))).toEqual(['#00c800', '#80e480']);
  expect(palette(px([128, 128, 128, 10], [5, 5, 5, 10]))).toBeNull();
});

it('presets: built-ins are valid looks and reset what they do not set', async () => {
  const { applyPreset, EQ_DEFAULTS, pickLook, PRESETS } = await import('./settings');
  expect(PRESETS.map((p) => p.name)).toEqual(['Neon', 'Retro LED', 'Minimal', 'Aurora', 'Rainbow Wave']);
  for (const p of PRESETS) for (const k of Object.keys(p.values)) expect(k in EQ_DEFAULTS).toBe(true);
  const patch = applyPreset({ name: 'x', values: { style: 'led' } });
  expect(patch.style).toBe('led');
  expect(patch.glow).toBe(EQ_DEFAULTS.glow);
  expect('length' in patch || 'myPresets' in patch || 'fps' in patch).toBe(false); // size and power stay yours
  expect('myPresets' in pickLook({ ...EQ_DEFAULTS, myPresets: [{ name: 'a', values: {} }] })).toBe(false);
});

describe('shrinking when silent', () => {
  it('waits out short silences, like the gap between songs', () => {
    expect(shrunkAt(1000, 2500)).toBe(false);
  });
  it('shrinks after two seconds of silence', () => {
    expect(shrunkAt(1000, 3000)).toBe(true);
  });
  it('grows back as soon as there is sound', () => {
    expect(shrunkAt(null, 99999)).toBe(false);
  });
});

it("ignores GlassBar's own volume tick, not the music around it", () => {
  // The tick plays at t = 1000 ms; the bars skip what the speakers play while it sounds.
  expect(duringBlip(1100, 1000)).toBe(true);
  expect(duringBlip(1350, 1000)).toBe(true);
  expect(duringBlip(1500, 1000)).toBe(false);
  expect(duringBlip(1100, -Infinity)).toBe(false); // no tick yet
});
