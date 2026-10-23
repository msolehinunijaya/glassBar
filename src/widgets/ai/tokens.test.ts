import { expect, it } from 'vitest';
import { tokens } from './tokens';

it('formats token counts compactly', () => {
  expect(tokens(950)).toBe('950');
  expect(tokens(1234)).toBe('1.2k');
  expect(tokens(12_340)).toBe('12k');
  expect(tokens(2_500_000)).toBe('2.5M');
});
