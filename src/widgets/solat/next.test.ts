import { describe, expect, it } from 'vitest';
import type { PrayerDay } from '../../lib/ipc';
import { countdown, nextPrayer } from './next';

const day = (date: string, base: number): PrayerDay => ({
  date, zone: 'WLY01', source: 'jakim', hijri: null,
  times: [['Imsak', base + 1], ['Subuh', base + 2], ['Syuruk', base + 3], ['Zohor', base + 4], ['Isyak', base + 5]],
});

describe('nextPrayer', () => {
  it('skips Syuruk and optional Imsak, then rolls into tomorrow', () => {
    const days = [day('d1', 0), day('d2', 100)];
    expect(nextPrayer(days, 0, false)?.name).toBe('Subuh');
    expect(nextPrayer(days, 0, true)?.name).toBe('Imsak');
    expect(nextPrayer(days, 2, false)?.name).toBe('Zohor');
    expect(nextPrayer(days, 5, false)).toEqual({ name: 'Subuh', at: 102 });
    expect(nextPrayer(days, 999, false)).toBeNull();
  });
});

describe('countdown', () => {
  it('shows seconds only in the last 10 minutes', () => {
    expect(countdown(9 * 60_000 + 5_000)).toBe('09:05');
    expect(countdown(2 * 3_600_000 + 5 * 60_000)).toBe('2h 05m');
    expect(countdown(25 * 60_000)).toBe('25m');
  });
});
