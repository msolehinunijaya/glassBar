import { describe, expect, it } from 'vitest';
import type { LyricLine } from '../../lib/ipc';
import { currentIndex, lineAt, lineProgress } from './lyrics';

const L: LyricLine[] = [
  { atMs: 10_000, text: 'First' },
  { atMs: 20_000, text: 'Second' },
  { atMs: 25_000, text: '' }, // a gap between verses
  { atMs: 30_000, text: 'Third' },
];

describe('currentIndex', () => {
  it('shows nothing before the first line', () => {
    expect(currentIndex(L, 0)).toBe(-1);
    expect(currentIndex(L, 9_999)).toBe(-1);
  });

  it('switches exactly on a line stamp', () => {
    expect(currentIndex(L, 10_000)).toBe(0);
    expect(currentIndex(L, 19_999)).toBe(0);
    expect(currentIndex(L, 20_000)).toBe(1);
  });

  it('keeps the last line up to the end of the song', () => {
    expect(currentIndex(L, 30_000)).toBe(3);
    expect(currentIndex(L, 9_999_999)).toBe(3);
  });

  it('holds the gap line, so the dock goes quiet between verses', () => {
    expect(currentIndex(L, 26_000)).toBe(2);
    expect(lineAt(L, currentIndex(L, 26_000))).toBe('');
  });

  it('copes with no lyrics at all', () => {
    expect(currentIndex([], 5_000)).toBe(-1);
    expect(lineAt([], -1)).toBe('');
  });

  it('agrees with a plain scan at every second of the song', () => {
    const slow = (ms: number) => { let r = -1; L.forEach((l, i) => { if (l.atMs <= ms) r = i; }); return r; };
    for (let ms = 0; ms <= 40_000; ms += 250) expect(currentIndex(L, ms)).toBe(slow(ms));
  });

  it('finds the right line in a long song', () => {
    const many: LyricLine[] = Array.from({ length: 500 }, (_, i) => ({ atMs: i * 1_000, text: `line ${i}` }));
    expect(currentIndex(many, 317_400)).toBe(317);
    expect(lineAt(many, currentIndex(many, 317_400))).toBe('line 317');
  });
});

describe('lineProgress', () => {
  it('runs 0 to 1 across a line', () => {
    expect(lineProgress(L, 0, 10_000, 40_000)).toBe(0);
    expect(lineProgress(L, 0, 15_000, 40_000)).toBe(0.5);
    expect(lineProgress(L, 0, 20_000, 40_000)).toBe(1);
  });

  it('measures the last line against the end of the song', () => {
    expect(lineProgress(L, 3, 35_000, 40_000)).toBe(0.5);
  });

  it('reads as full when there is nothing to measure against', () => {
    expect(lineProgress(L, 3, 35_000, 0)).toBe(1);
    expect(lineProgress([{ atMs: 0, text: 'x' }], 0, 0, 0)).toBe(1);
  });

  it('never leaves 0..1, whatever the position', () => {
    expect(lineProgress(L, 0, 0, 40_000)).toBe(0);
    expect(lineProgress(L, 0, 99_999, 40_000)).toBe(1);
  });

  it('is 0 with no line selected', () => {
    expect(lineProgress(L, -1, 5_000, 40_000)).toBe(0);
    expect(lineProgress(L, 99, 5_000, 40_000)).toBe(0);
  });
});
