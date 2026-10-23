import { describe, expect, it } from 'vitest';
import { toggleCheck } from './options';

describe('toggleCheck (multi-checkbox with a minimum)', () => {
  it('unchecks and checks normally', () => {
    expect(toggleCheck({ cpu: true, ram: true }, 'ram', false, 1)).toEqual({ cpu: true, ram: false });
    expect(toggleCheck({ cpu: true, ram: false }, 'ram', true, 1)).toEqual({ cpu: true, ram: true });
  });
  it('refuses to uncheck the last checked item', () => {
    expect(toggleCheck({ cpu: true, ram: false, net: false }, 'cpu', false, 1)).toEqual({ cpu: true, ram: false, net: false });
  });
});
