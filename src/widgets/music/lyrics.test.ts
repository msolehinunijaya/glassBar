import { describe, expect, it } from 'vitest';
import type { LyricLine } from '../../lib/ipc';
import { currentIndex, fitLine, inGap, lineAt, lineProgress } from './lyrics';

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

describe('fitLine', () => {
  // A fake font: every character is half the font size wide.
  const m = (text: string) => (px: number) => text.length * px * 0.5;
  it('keeps the chosen size when the line fits', () => {
    expect(fitLine(m('short line'), 200, 12, true)).toEqual({ size: 12, wrap: false });
  });
  it('steps down one size before wrapping', () => {
    // 34 chars: 204 px at 12, 187 px at 11
    expect(fitLine(m('x'.repeat(34)), 200, 12, true)).toEqual({ size: 11, wrap: false });
  });
  it('wraps onto two lines when one size smaller still does not fit', () => {
    expect(fitLine(m('x'.repeat(60)), 200, 12, true)).toEqual({ size: 11, wrap: true });
  });
  it('never wraps when wrapping is not allowed (karaoke)', () => {
    expect(fitLine(m('x'.repeat(60)), 200, 12, false)).toEqual({ size: 11, wrap: false });
  });
  it('never goes below 10 px', () => {
    expect(fitLine(m('x'.repeat(60)), 200, 10, true).size).toBe(10);
  });
});

describe('inGap', () => {
  const L = [{ atMs: 0, text: '' }, { atMs: 8000, text: 'one' }, { atMs: 10000, text: '' }, { atMs: 12000, text: 'two' }];
  it('is a gap when the silence lasts over 5 s', () => {
    expect(inGap(L, 0, 60000)).toBe(true);
  });
  it('is not a gap for a short pause', () => {
    expect(inGap(L, 2, 60000)).toBe(false);
  });
  it('is not a gap while words show', () => {
    expect(inGap(L, 1, 60000)).toBe(false);
  });
  it('counts the intro before the first line', () => {
    expect(inGap([{ atMs: 9000, text: 'hi' }], -1, 60000)).toBe(true);
    expect(inGap([], -1, 60000)).toBe(false);
  });
});
