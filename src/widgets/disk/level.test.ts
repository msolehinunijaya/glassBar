import { describe, expect, it } from 'vitest';
import { bytes, letter, level, pickDrives, usedPct } from './level';

const d = (mount: string, total: number, available: number) => ({ mount, label: '', total, available, removable: false, fs: 'NTFS' });

describe('disk level', () => {
  it('computes used %', () => expect(usedPct(d('C:\\', 100, 32))).toBe(68));
  it('bands by thresholds', () => {
    expect(level(50, 80, 90)).toBe('normal');
    expect(level(80, 80, 90)).toBe('high');
    expect(level(95, 80, 90)).toBe('critical');
  });
  it('formats sizes', () => {
    expect(bytes(512 * 1024 ** 3)).toBe('512 GB');
    expect(bytes(1.5 * 1024 ** 4)).toBe('1.5 TB');
  });
  it('drive letter', () => expect(letter('D:\\')).toBe('D:'));
  it('defaults to C: (or the first drive) and skips unplugged picks', () => {
    const all = [d('C:\\', 1, 1), d('E:\\', 1, 1)];
    expect(pickDrives(all, []).map((x) => x.mount)).toEqual(['C:\\']);
    expect(pickDrives(all, ['E:\\', 'Z:\\']).map((x) => x.mount)).toEqual(['E:\\']);
    expect(pickDrives([d('D:\\', 1, 1)], []).map((x) => x.mount)).toEqual(['D:\\']);
  });
});
