import { describe, expect, it } from 'vitest';
import { fmtTemp, isHot } from './temp';

describe('fmtTemp', () => {
  it('shows N/A when the PC has no sensor', () => expect(fmtTemp(null, 'c')).toBe('N/A'));
  it('rounds °C', () => expect(fmtTemp(52.4, 'c')).toBe('52°C'));
  it('converts to °F', () => expect(fmtTemp(50, 'f')).toBe('122°F'));
});

describe('isHot', () => {
  it('compares in °C regardless of display unit', () => {
    expect(isHot(85, 80)).toBe(true);
    expect(isHot(79.9, 80)).toBe(false);
    expect(isHot(null, 80)).toBe(false);
  });
});
