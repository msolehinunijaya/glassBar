import { describe, expect, it } from 'vitest';
import { compass, daylight, sunArc, themeOf } from './sun';

const H = 3_600_000;
describe('sunArc', () => {
  it('is halfway at solar noon', () => expect(sunArc(12 * H, 6 * H, 18 * H)).toEqual({ frac: 0.5, phase: 'day' }));
  it('clamps before sunrise and after sunset', () => {
    expect(sunArc(5 * H, 6 * H, 18 * H)).toEqual({ frac: 0, phase: 'before' });
    expect(sunArc(20 * H, 6 * H, 18 * H)).toEqual({ frac: 1, phase: 'after' });
  });
  it('reports no sun data (polar day/night)', () => expect(sunArc(1, null, null).phase).toBe('none'));
});

describe('daylight', () => {
  it('formats hours and minutes', () => expect(daylight(6 * H, 18 * H + 8 * 60_000)).toBe('12h 8m'));
  it('is null without data', () => expect(daylight(null, 5)).toBeNull());
});

describe('compass', () => {
  it('maps degrees to 8 points', () => {
    expect(compass(0)).toBe('N');
    expect(compass(44)).toBe('NE');
    expect(compass(180)).toBe('S');
    expect(compass(350)).toBe('N');
  });
});

describe('themeOf', () => {
  it('differs by condition and day/night', () => {
    expect(themeOf('sun', true)).not.toBe(themeOf('rain', true));
    expect(themeOf('sun', true)).not.toBe(themeOf('moon', false));
  });
});
