import { describe, expect, it } from 'vitest';
import type { PrayerDay } from '../../lib/ipc';
import { prayerState } from './state';

const MIN = 60_000;
const day: PrayerDay = {
  date: '2026-10-23', zone: 'WLY01', source: 'jakim', hijri: null,
  times: [['Imsak', 0], ['Subuh', 100 * MIN], ['Syuruk', 170 * MIN], ['Zohor', 500 * MIN], ['Asar', 700 * MIN], ['Maghrib', 900 * MIN], ['Isyak', 1000 * MIN]],
};

describe('prayerState', () => {
  it('counts down with progress from the previous prayer to the next', () => {
    const s = prayerState([day], 600 * MIN, null, false)!;
    expect(s.next.name).toBe('Asar');
    expect(s.phase).toBe('countdown');
    expect(s.progress).toBeCloseTo(0.5); // Zohor 500 -> Asar 700, now 600
  });
  it("keeps a progress fill between midnight and Subuh (last night's Isyak isn't loaded)", () => {
    // 50 min after midnight; last night's Isyak ≈ today's Isyak (1000) - 1 day = -440
    const s = prayerState([day], 50 * MIN, null, false)!;
    expect(s.next.name).toBe('Subuh');
    expect(s.progress).toBeCloseTo((50 + 440) / (100 + 440));
    expect(s.reached).toBeNull();
    // Same with tomorrow loaded too (the real data): still today's Isyak, not tomorrow's Subuh.
    const D = 1440 * MIN;
    const tomorrow: PrayerDay = { ...day, date: '2026-10-24', times: day.times.map(([n, t]) => [n, t + D]) };
    expect(prayerState([day, tomorrow], 50 * MIN, null, false)!.progress).toBeCloseTo((50 + 440) / (100 + 440));
  });
  it('is "approaching" in the last 15 minutes', () => {
    expect(prayerState([day], 690 * MIN, null, false)!.phase).toBe('approaching');
  });
  it('is "reached" after a prayer time until acknowledged', () => {
    const s = prayerState([day], 705 * MIN, null, false)!;
    expect(s.phase).toBe('reached');
    expect(s.reached?.key).toBe('2026-10-23:Asar');
    const acked = prayerState([day], 705 * MIN, '2026-10-23:Asar', false)!;
    expect(acked.phase).toBe('countdown');
    expect(acked.next.name).toBe('Maghrib');
  });
  it('does not flash for a prayer that passed more than 30 minutes ago', () => {
    expect(prayerState([day], 740 * MIN, null, false)!.phase).toBe('countdown');
  });
});
