import { describe, expect, it } from 'vitest';
import { ago, cutMiddle, fmtSize, kindOf, when } from './format';

describe('cutMiddle', () => {
  it('keeps short names whole', () => expect(cutMiddle('report.pdf', 22)).toBe('report.pdf'));
  it('cuts the middle and keeps the extension', () => {
    const s = cutMiddle('Invoice_October_statement_2026.pdf', 22);
    expect(s.length).toBeLessThanOrEqual(22);
    expect(s.endsWith('.pdf')).toBe(true);
    expect(s.startsWith('Invoice')).toBe(true);
    expect(s).toContain('…');
  });
  it('treats a long "extension" as part of the name', () => {
    const s = cutMiddle('archive.thisisnotanextension', 12);
    expect(s.length).toBeLessThanOrEqual(12);
    expect(s).toContain('…');
  });
});

describe('fmtSize', () => {
  it('formats B / KB / MB / GB', () => {
    expect(fmtSize(0)).toBe('0 B');
    expect(fmtSize(512)).toBe('512 B');
    expect(fmtSize(1536)).toBe('1.5 KB');
    expect(fmtSize(23 * 1024 * 1024)).toBe('23 MB');
    expect(fmtSize(2.4 * 1024 ** 3)).toBe('2.4 GB');
  });
});

describe('time text', () => {
  const now = new Date(2026, 9, 24, 14, 30).getTime();
  it('ago', () => {
    expect(ago(now - 20_000, now)).toBe('just now');
    expect(ago(now - 2 * 60_000, now)).toBe('2 min ago');
    expect(ago(now - 3 * 3_600_000, now)).toBe('3 h ago');
    expect(ago(now - 30 * 3_600_000, now)).toBe('yesterday');
    expect(ago(now - 4 * 86_400_000, now)).toBe('4 days ago');
  });
  it('when: today with the time, yesterday, else the date', () => {
    expect(when(new Date(2026, 9, 24, 14, 5).getTime(), now, 'en-US')).toBe('Today 2:05 PM');
    expect(when(new Date(2026, 9, 23, 9, 0).getTime(), now, 'en-US')).toBe('Yesterday');
    expect(when(new Date(2026, 9, 2).getTime(), now, 'en-US')).toBe('Oct 2');
  });
});

describe('kindOf (filters)', () => {
  it('groups by extension', () => {
    expect(kindOf('a.PDF')).toBe('documents');
    expect(kindOf('photo.jpeg')).toBe('images');
    expect(kindOf('clip.mp4')).toBe('videos');
    expect(kindOf('stuff.7z')).toBe('archives');
    expect(kindOf('setup.exe')).toBe('installers');
    expect(kindOf('noext')).toBe('other');
    expect(kindOf('weird.xyz')).toBe('other');
  });
});
