import { describe, expect, it } from 'vitest';
import { optionId, toggleCheck } from './options';

describe('toggleCheck (multi-checkbox with a minimum)', () => {
  it('unchecks and checks normally', () => {
    expect(toggleCheck({ cpu: true, ram: true }, 'ram', false, 1)).toEqual({ cpu: true, ram: false });
    expect(toggleCheck({ cpu: true, ram: false }, 'ram', true, 1)).toEqual({ cpu: true, ram: true });
  });
  it('refuses to uncheck the last checked item', () => {
    expect(toggleCheck({ cpu: true, ram: false, net: false }, 'cpu', false, 1)).toEqual({ cpu: true, ram: false, net: false });
  });
});

describe('optionId', () => {
  it('tells apart rows that share a label (the two Equalizer "Colour" rows)', () => {
    expect(optionId({ key: 'colorMode', label: 'Colour' })).not.toBe(optionId({ key: 'color1', label: 'Colour' }));
  });
  it('falls back to the label for rows without a setting key', () => {
    expect(optionId({ label: 'Show' })).toBe('label:Show');
  });
});
