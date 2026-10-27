import { describe, expect, it } from 'vitest';
import { fmtWater, progressOf } from './units';

describe('water units', () => {
  it('writes ml with thousands separators', () => {
    expect(fmtWater(750)).toBe('750 ml');
    expect(fmtWater(1250)).toBe('1,250 ml');
  });
  it('writes US fluid ounces', () => {
    expect(fmtWater(750, 'oz')).toBe('25 fl oz');
  });
  it('keeps progress between empty and full', () => {
    expect(progressOf(500, 2000)).toBe(0.25);
    expect(progressOf(2500, 2000)).toBe(1);
    expect(progressOf(100, 0)).toBe(0);
  });
});
