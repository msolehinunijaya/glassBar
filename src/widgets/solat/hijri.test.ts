import { describe, expect, it } from 'vitest';
import type { PrayerDay } from '../../lib/ipc';
import { hijriLabel, hijriNow, parseHijri } from './hijri';
import { DHIKR, phraseOf, stepsOf } from './tasbih';

const day = (date: string, hijri: string | null, maghrib: number): PrayerDay => ({ date, zone: 'WLY01', source: 'jakim', hijri, times: [['Maghrib', maghrib]] });

describe('Hijri date', () => {
  it('reads JAKIM and Aladhan formats', () => {
    expect(parseHijri('1448-05-12')).toEqual({ y: 1448, m: 5, d: 12 });
    expect(parseHijri('12-05-1448')).toEqual({ y: 1448, m: 5, d: 12 });
    expect(parseHijri('soon')).toBe(null);
  });
  it('names the month in Malay', () => {
    expect(hijriLabel({ y: 1448, m: 5, d: 12 })).toBe('12 Jamadilawal 1448 H');
    expect(hijriLabel({ y: 1448, m: 10, d: 1 })).toBe('1 Syawal 1448 H');
  });
  it('uses JAKIM before Maghrib and the next Hijri day after it', () => {
    const maghrib = new Date(2026, 9, 30, 19, 10).getTime();
    const days = [day('2026-10-30', '1448-05-12', maghrib), day('2026-10-31', '1448-05-13', maghrib + 86_400_000)];
    expect(hijriNow(days, new Date(2026, 9, 30, 12).getTime())).toMatchObject({ label: '12 Jamadilawal 1448 H', estimate: false, afterMaghrib: false });
    expect(hijriNow(days, new Date(2026, 9, 30, 20).getTime())).toMatchObject({ label: '13 Jamadilawal 1448 H', estimate: false, afterMaghrib: true });
  });
  it('falls back to the calculated date, marked an estimate', () => {
    const h = hijriNow([], new Date(2026, 9, 30, 12).getTime());
    expect(h.estimate).toBe(true);
    expect(h.label).toMatch(/^\d{1,2} Jamadilawal 1448 H$/);
  });
});

describe('Tasbih', () => {
  it('knows its phrases and targets', () => {
    expect(DHIKR.map((d) => d.key)).toContain('tasbih_fatimah');
    expect(stepsOf('tasbih_fatimah', 99)).toEqual([33, 33, 34]);
    expect(stepsOf('subhanallah', 99)).toEqual([99]);
    expect(phraseOf('tasbih_fatimah', 1)).toBe('Alhamdulillah');
    expect(phraseOf('astaghfirullah', 0)).toBe('Astaghfirullah');
  });
});
