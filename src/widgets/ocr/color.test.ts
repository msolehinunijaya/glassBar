import { describe, expect, it } from 'vitest';
import { formatColor, pushColor, textOn, toHex } from './color';

describe('colours', () => {
  it('writes hex in capitals', () => {
    expect(toHex(59, 130, 246)).toBe('#3B82F6');
    expect(toHex(0, 0, 0)).toBe('#000000');
  });
  it('formats as hex, rgb and hsl', () => {
    expect(formatColor('#3B82F6', 'hex')).toBe('#3B82F6');
    expect(formatColor('#3B82F6', 'rgb')).toBe('rgb(59, 130, 246)');
    expect(formatColor('#3B82F6', 'hsl')).toBe('hsl(217, 91%, 60%)');
    expect(formatColor('#808080', 'hsl')).toBe('hsl(0, 0%, 50%)');
  });
  it('keeps the newest first, no repeats, at most 16', () => {
    expect(pushColor(['#000000', '#FFFFFF'], '#FFFFFF')).toEqual(['#FFFFFF', '#000000']);
    const many = Array.from({ length: 16 }, (_, i) => toHex(i, i, i));
    expect(pushColor(many, '#ABCDEF')).toHaveLength(16);
    expect(pushColor(many, '#ABCDEF')[0]).toBe('#ABCDEF');
  });
  it('picks readable text on a swatch', () => {
    expect(textOn('#FFFFFF')).toBe('#000');
    expect(textOn('#1E1E1E')).toBe('#fff');
  });
});
