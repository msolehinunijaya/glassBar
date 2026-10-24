import { describe, expect, it } from 'vitest';
import { dayState, sunTimes } from './solar';

// Minutes since local midnight in a time zone, for comparing with published times.
const localMin = (ms: number, tz: string) => {
  const p = Object.fromEntries(new Intl.DateTimeFormat('en-US', { timeZone: tz, hourCycle: 'h23', hour: 'numeric', minute: 'numeric' }).formatToParts(new Date(ms)).map((x) => [x.type, +x.value]));
  return p.hour * 60 + p.minute;
};
const near = (got: number, h: number, m: number, tol = 4) => expect(Math.abs(got - (h * 60 + m))).toBeLessThanOrEqual(tol);

describe('sunTimes (offline sunrise equation)', () => {
  it('Kuala Lumpur, 23 Oct 2026: about 06:57 and 18:59', () => {
    const s = sunTimes(3.139, 101.687, 2026, 10, 23);
    near(localMin(s.rise!, 'Asia/Kuala_Lumpur'), 6, 57);
    near(localMin(s.set!, 'Asia/Kuala_Lumpur'), 18, 59);
    expect(new Date(s.rise!).getUTCDate()).toBe(22); // 06:57 local on the 23rd = 22:57 UTC on the 22nd
  });
  it('London, midsummer 2026: about 04:43 and 21:21 BST', () => {
    const s = sunTimes(51.507, -0.128, 2026, 6, 21);
    near(localMin(s.rise!, 'Europe/London'), 4, 43);
    near(localMin(s.set!, 'Europe/London'), 21, 21);
  });
  it('Auckland (across the date line) gets its own local day', () => {
    const s = sunTimes(-36.85, 174.76, 2026, 10, 23);
    near(localMin(s.rise!, 'Pacific/Auckland'), 6, 26, 8);
    near(localMin(s.set!, 'Pacific/Auckland'), 19, 44, 8);
    // Sunrise in Auckland on the 23rd local is still the 22nd in UTC.
    expect(new Date(s.rise!).getUTCDate()).toBe(22);
  });
  it('polar night and midnight sun (Tromsø)', () => {
    expect(sunTimes(69.65, 18.96, 2026, 12, 21).polar).toBe('night');
    expect(sunTimes(69.65, 18.96, 2026, 6, 21).polar).toBe('day');
  });
});

describe('dayState', () => {
  const KL = { lat: 3.139, lon: 101.687 };
  const klAt = (h: number, m = 0) => Date.UTC(2026, 9, 23, h - 8, m); // KL is UTC+8
  it('day, night and twilight around sunrise/sunset', () => {
    expect(dayState(klAt(12), 'Asia/Kuala_Lumpur', KL).state).toBe('day');
    expect(dayState(klAt(23), 'Asia/Kuala_Lumpur', KL).state).toBe('night');
    expect(dayState(klAt(7, 10), 'Asia/Kuala_Lumpur', KL).state).toBe('sunrise');
    expect(dayState(klAt(19, 0), 'Asia/Kuala_Lumpur', KL).state).toBe('sunset');
  });
  it('names the next sunrise at night', () => {
    const s = dayState(klAt(23), 'Asia/Kuala_Lumpur', KL);
    expect(new Date(s.nextRise!).getUTCDate()).toBe(23); // tomorrow's 07:0x local = 23:0x UTC on the 23rd
    near(localMin(s.nextRise!, 'Asia/Kuala_Lumpur'), 6, 57);
  });
  it('polar day/night: sun or moon all day, no errors', () => {
    const t = Date.UTC(2026, 11, 21, 12);
    expect(dayState(t, 'Europe/Oslo', { lat: 69.65, lon: 18.96 }).state).toBe('night');
    expect(dayState(Date.UTC(2026, 5, 21, 0), 'Europe/Oslo', { lat: 69.65, lon: 18.96 }).state).toBe('day');
  });
  it('without coordinates: 6 AM to 6 PM local is day', () => {
    expect(dayState(Date.UTC(2026, 9, 23, 5 - 9), 'Asia/Tokyo').state).toBe('night'); // 05:00 Tokyo
    expect(dayState(Date.UTC(2026, 9, 23, 6 - 9), 'Asia/Tokyo').state).toBe('day'); // 06:00
    expect(dayState(Date.UTC(2026, 9, 23, 18 - 9), 'Asia/Tokyo').state).toBe('night'); // 18:00
  });
});
