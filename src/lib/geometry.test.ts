import { describe, expect, it } from 'vitest';
import { reorder, snap, indexAt } from './geometry';

const mons = [{ x: 0, y: 0, w: 1920, h: 1080 }, { x: 1920, y: 0, w: 1920, h: 1080 }];

describe('snap', () => {
  it('docks to the nearest edge within threshold', () => {
    expect(snap({ x: 400, y: 1080 - 60, w: 1000, h: 48 }, mons, 24)).toMatchObject({ monitor: 0, edge: 'bottom' });
    expect(snap({ x: 400, y: 10, w: 1000, h: 48 }, mons, 24)).toMatchObject({ monitor: 0, edge: 'top' });
  });
  it('floats when no edge is close', () => {
    expect(snap({ x: 400, y: 500, w: 1000, h: 48 }, mons, 24)).toMatchObject({ monitor: 0, edge: null });
  });
  it('picks the monitor holding the window center', () => {
    expect(snap({ x: 2000, y: 1030, w: 1000, h: 48 }, mons, 24)).toMatchObject({ monitor: 1, edge: 'bottom' });
  });
  it('docks to a side edge', () => {
    expect(snap({ x: 1920 + 1920 - 50, y: 200, w: 48, h: 700 }, mons, 24)).toMatchObject({ monitor: 1, edge: 'right' });
  });
  it('falls back to the nearest monitor when the center is off-screen', () => {
    expect(snap({ x: -900, y: 10, w: 1000, h: 48 }, mons, 24).monitor).toBe(0);
  });
});

describe('reorder', () => {
  it('moves an item forward and back', () => {
    expect(reorder(['a', 'b', 'c', 'd'], 0, 2)).toEqual(['b', 'c', 'a', 'd']);
    expect(reorder(['a', 'b', 'c', 'd'], 3, 0)).toEqual(['d', 'a', 'b', 'c']);
  });
  it('does not mutate and clamps the target', () => {
    const src = ['a', 'b'];
    expect(reorder(src, 0, 9)).toEqual(['b', 'a']);
    expect(src).toEqual(['a', 'b']);
  });
});

describe('indexAt', () => {
  it('finds the slot a pointer is over from item centers', () => {
    const centers = [50, 150, 250];
    expect(indexAt(centers, 10)).toBe(0);
    expect(indexAt(centers, 120)).toBe(1);
    expect(indexAt(centers, 999)).toBe(2);
  });
});

describe('snap: drag past the edge to auto-hide', () => {
  const one = [{ x: 0, y: 0, w: 1920, h: 1080 }];
  it('hides when the bar centre goes below the bottom edge', () => {
    expect(snap({ x: 400, y: 1060, w: 1000, h: 60 }, one, 24)).toEqual({ monitor: 0, edge: 'bottom', hide: true });
  });
  it('hides past the top and side edges too', () => {
    expect(snap({ x: 400, y: -40, w: 1000, h: 60 }, one, 24)).toMatchObject({ edge: 'top', hide: true });
    expect(snap({ x: -40, y: 200, w: 60, h: 700 }, one, 24)).toMatchObject({ edge: 'left', hide: true });
  });
  it('does not hide when merely near the edge', () => {
    expect(snap({ x: 400, y: 1010, w: 1000, h: 60 }, one, 24)).toEqual({ monitor: 0, edge: 'bottom', hide: false });
  });
});
