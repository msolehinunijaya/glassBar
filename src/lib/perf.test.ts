import { describe, expect, it } from 'vitest';
import { fpsCap } from './perf';
describe('fpsCap', () => {
  it('Low is 15 whatever the setting', () => { expect(fpsCap('low', 60)).toBe(15); expect(fpsCap('low', 30)).toBe(15); });
  it('Normal caps at 30', () => { expect(fpsCap('normal', 60)).toBe(30); expect(fpsCap('normal', 30)).toBe(30); });
  it('High is the user setting', () => { expect(fpsCap('high', 60)).toBe(60); expect(fpsCap('high', 30)).toBe(30); });
});
