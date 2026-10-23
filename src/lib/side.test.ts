import { describe, expect, it } from 'vitest';
import { loadTone, rateShort } from './side';

describe('side tiles', () => {
  it('shortens rates to fit a side tile', () => {
    expect(rateShort(0)).toBe('0');
    expect(rateShort(12)).toBe('12');
    expect(rateShort(978 * 1024)).toBe('978K');
    expect(rateShort(1.1 * 1024 * 1024)).toBe('1.1M');
    expect(rateShort(24 * 1024 * 1024)).toBe('24M');
    expect(rateShort(1000)).toBe('1.0K');
  });
  it('colours load by threshold', () => {
    expect(loadTone(10)).toBe('var(--accent)');
    expect(loadTone(60)).toBe('#ff9f0a');
    expect(loadTone(90)).toBe('#ff453a');
  });
});
