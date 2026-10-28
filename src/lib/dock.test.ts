import { describe, expect, it } from 'vitest';
import { isDockWindow, isMainDock } from './dock';

describe('dock windows', () => {
  it('knows the main dock and the copies on other monitors', () => {
    expect(isDockWindow('bar')).toBe(true);
    expect(isDockWindow('bar-DISPLAY2')).toBe(true);
    expect(isDockWindow('settings')).toBe(false);
    expect(isMainDock('bar')).toBe(true);
    expect(isMainDock('bar-DISPLAY2')).toBe(false);
  });
});
