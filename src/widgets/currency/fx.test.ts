import { describe, expect, it } from 'vitest';
import { convert, fmtAmount, pairKey, parsePair } from './fx';

describe('fx', () => {
  const rates = { USD: 1, MYR: 4.2, JPY: 150 };
  it('converts from the base', () => expect(convert(10, rates, 'MYR')).toBeCloseTo(42));
  it('is null for unknown currency', () => expect(convert(1, rates, 'XXX')).toBeNull());
  it('formats with sensible precision', () => {
    expect(fmtAmount(4.2)).toBe('4.20');
    expect(fmtAmount(0.004567)).toBe('0.004567');
    expect(fmtAmount(15000)).toBe('15,000.00');
  });
  it('round-trips favourite pairs', () => {
    expect(pairKey('usd', 'myr')).toBe('USD>MYR');
    expect(parsePair('USD>MYR')).toEqual(['USD', 'MYR']);
    expect(parsePair('bad')).toBeNull();
  });
});
