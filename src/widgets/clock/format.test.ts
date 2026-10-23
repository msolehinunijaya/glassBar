import { describe, expect, it } from 'vitest';
import { formatDate, formatTime, isValidPattern, type DateOpts, type TimeOpts } from './format';

// Fri 23 Oct 2026, local time (tests use a fixed locale so names are predictable).
const at = (h: number, m = 5, s = 9) => new Date(2026, 9, 23, h, m, s);
const t24: TimeOpts = { h12: false, seconds: false, leadingZero: true, ampm: 'PM' };
const t12: TimeOpts = { h12: true, seconds: false, leadingZero: false, ampm: 'PM' };
const show = (r: { main: string; ampm: string }) => (r.ampm ? `${r.main} ${r.ampm}` : r.main);

describe('formatTime', () => {
  it('24-hour, with and without seconds', () => {
    expect(show(formatTime(at(14), t24))).toBe('14:05');
    expect(show(formatTime(at(14), { ...t24, seconds: true }))).toBe('14:05:09');
    expect(show(formatTime(at(2), t24))).toBe('02:05');
    expect(show(formatTime(at(2), { ...t24, leadingZero: false }))).toBe('2:05');
  });
  it('12-hour with AM/PM, with and without seconds', () => {
    expect(show(formatTime(at(14), t12))).toBe('2:05 PM');
    expect(show(formatTime(at(14), { ...t12, seconds: true }))).toBe('2:05:09 PM');
    expect(show(formatTime(at(9), { ...t12, leadingZero: true }))).toBe('09:05 AM');
  });
  it('midnight and noon: 12 AM and 12 PM', () => {
    expect(show(formatTime(at(0, 0), t12))).toBe('12:00 AM');
    expect(show(formatTime(at(12, 0), t12))).toBe('12:00 PM');
    expect(show(formatTime(at(0, 0), t24))).toBe('00:00');
    expect(show(formatTime(at(12, 0), t24))).toBe('12:00');
  });
  it('AM/PM styles', () => {
    expect(formatTime(at(14), { ...t12, ampm: 'pm' }).ampm).toBe('pm');
    expect(formatTime(at(9), { ...t12, ampm: 'P.M.' }).ampm).toBe('A.M.');
    expect(formatTime(at(14), { ...t12, ampm: 'p.m.' }).ampm).toBe('p.m.');
  });
  it('another time zone', () => {
    const utc = new Date(Date.UTC(2026, 9, 23, 6, 30));
    expect(formatTime(utc, t24, 'Asia/Tokyo').main).toBe('15:30');
  });
});

describe('formatDate presets', () => {
  const d = at(14);
  const o = (style: DateOpts['style'], extra: Partial<DateOpts> = {}): DateOpts => ({ style, sep: '/', weekday: 'none', pattern: '', locale: 'en-GB', ...extra });
  it('each preset', () => {
    expect(formatDate(d, o('short'))).toBe('23/10');
    expect(formatDate(d, o('dmy'))).toBe('23/10/2026');
    expect(formatDate(d, o('mdy'))).toBe('10/23/2026');
    expect(formatDate(d, o('iso'))).toBe('2026-10-23');
    expect(formatDate(d, o('medium'))).toBe('23 Oct 2026');
    expect(formatDate(d, o('weekday'))).toBe('Fri, 23 Oct');
    expect(formatDate(d, o('long'))).toBe('Friday, 23 October 2026');
  });
  it('separator and weekday options', () => {
    expect(formatDate(d, o('dmy', { sep: '.' }))).toBe('23.10.2026');
    expect(formatDate(d, o('mdy', { sep: '-' }))).toBe('10-23-2026');
    expect(formatDate(d, o('iso', { sep: '.' }))).toBe('2026-10-23'); // ISO always uses -
    expect(formatDate(d, o('dmy', { weekday: 'short' }))).toBe('Fri, 23/10/2026');
    expect(formatDate(d, o('medium', { weekday: 'full' }))).toBe('Friday, 23 Oct 2026');
  });
  it('names come from the locale, not hard-coded English', () => {
    expect(formatDate(d, o('long', { locale: 'ms-MY' }))).toBe('Jumaat, 23 Oktober 2026');
  });
});

describe('custom patterns', () => {
  const d = new Date(2026, 2, 5); // Thu 5 Mar 2026
  const c = (pattern: string) => formatDate(d, { style: 'custom', sep: '/', weekday: 'none', pattern, locale: 'en-GB' });
  it('every token', () => {
    expect(c('D DD M MM YY YYYY')).toBe('5 05 3 03 26 2026');
    expect(c('ddd dddd MMM MMMM')).toBe('Thu Thursday Mar March');
    expect(c('dddd, D MMMM')).toBe('Thursday, 5 March');
    expect(c('[Week of] D/M')).toBe('Week of 5/3');
  });
  it('an invalid pattern is detected and falls back instead of breaking', () => {
    expect(isValidPattern('Q MM')).toBe(false);
    expect(isValidPattern('')).toBe(false);
    expect(isValidPattern('DD/MM')).toBe(true);
    expect(c('Q MM')).toBe('Thu, 5 Mar'); // the default "With weekday" style
  });
});
