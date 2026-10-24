import { describe, expect, it } from 'vitest';
import { diffLabel, offsetMinutes, zonesOf } from './zones';

describe('world clock helpers', () => {
  it('reads a zone offset, including half-hour zones and DST', () => {
    const jan = new Date('2026-01-15T12:00:00Z'), jul = new Date('2026-07-15T12:00:00Z');
    expect(offsetMinutes('Asia/Kuala_Lumpur', jan)).toBe(480);
    expect(offsetMinutes('Asia/Kolkata', jan)).toBe(330);
    expect(offsetMinutes('Europe/London', jan)).toBe(0);
    expect(offsetMinutes('Europe/London', jul)).toBe(60); // British Summer Time
    expect(offsetMinutes('UTC', jan)).toBe(0);
  });
  it('labels the difference from local time', () => {
    expect(diffLabel(480, 480)).toBe('Same time');
    expect(diffLabel(480, 0)).toBe('8h behind');
    expect(diffLabel(480, 540)).toBe('1h ahead');
    expect(diffLabel(480, 330)).toBe('2h 30m behind');
  });
  it('turns the old single "second time zone" into the list', () => {
    expect(zonesOf({ zones: [], secondTz: 'Asia/Tokyo' })).toEqual([{ tz: 'Asia/Tokyo', label: 'Tokyo' }]);
    expect(zonesOf({ zones: [{ tz: 'UTC', label: 'UTC' }], secondTz: 'Asia/Tokyo' })).toEqual([{ tz: 'UTC', label: 'UTC' }]);
  });
});
