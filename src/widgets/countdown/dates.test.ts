import { describe, expect, it } from 'vitest';
import { left, nextCal, nextYearly, occurrence, rollOver, ymd, type Countdown } from './dates';

const D = (y: number, m: number, d: number, h = 0, min = 0) => new Date(y, m - 1, d, h, min);
const base: Countdown = { id: 'a', name: 'X', date: '2026-11-02', time: null, emoji: '🎯', repeat: 'none', rule: null, created: D(2026, 10, 1).getTime(), remindDayBefore: false };

describe('next dates', () => {
  it('finds the next fixed day', () => {
    expect(ymd(nextYearly('08-31', D(2026, 10, 30)))).toBe('2027-08-31');
    expect(ymd(nextYearly('12-25', D(2026, 10, 30)))).toBe('2026-12-25');
    expect(ymd(nextYearly('10-30', D(2026, 10, 30)))).toBe('2026-10-30');
  });
  it('finds 1 Syawal 1448 (Umm al-Qura) in March 2027', () => {
    const d = nextCal('islamic-umalqura', 10, 1, D(2026, 10, 30));
    expect(d.getFullYear()).toBe(2027);
    expect(d.getMonth()).toBe(2);
    expect([9, 10]).toContain(d.getDate());
  });
  it('finds Chinese New Year 2027 in early February', () => {
    const d = nextCal('chinese', 1, 1, D(2026, 10, 30));
    expect(ymd(d).slice(0, 8)).toBe('2027-02-');
    expect([6, 7]).toContain(d.getDate());
  });
});

describe('time left', () => {
  it('counts days and weeks', () => {
    const now = D(2026, 10, 30, 10);
    const t = occurrence({ ...base, date: '2027-03-11' }, now);
    expect(left(t, false, now)).toMatchObject({ big: '132 days', small: '18 weeks 6 days', short: '132d', state: 'future' });
  });
  it('says tomorrow and today for whole days', () => {
    const now = D(2026, 11, 1, 22);
    expect(left(occurrence(base, now), false, now)).toMatchObject({ big: 'Tomorrow', short: '1d' });
    const on = D(2026, 11, 2, 15);
    expect(left(occurrence(base, on), false, on)).toMatchObject({ big: 'Today', state: 'today' });
  });
  it('counts hours and minutes to a set time', () => {
    const c = { ...base, time: '18:00' };
    const now = D(2026, 11, 2, 12, 40);
    expect(left(occurrence(c, now), true, now)).toMatchObject({ big: '5h 20m', short: '5h' });
    const close = D(2026, 11, 2, 17, 18);
    expect(left(occurrence(c, close), true, close)).toMatchObject({ big: '42m', short: '42m' });
  });
  it('is done after the day, and repeats roll to the next year', () => {
    const later = D(2026, 11, 3, 9);
    expect(left(occurrence(base, later), false, later).state).toBe('done');
    const yearly = { ...base, repeat: 'yearly' as const, date: '2026-08-31' };
    expect(rollOver(yearly, later).date).toBe('2027-08-31');
    expect(rollOver(base, later).date).toBe('2026-11-02');
  });
});
