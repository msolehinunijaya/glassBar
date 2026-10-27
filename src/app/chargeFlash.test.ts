import { describe, expect, it } from 'vitest';
import { FLASH, FLASH_MS, fillOf, flashText, flashTone } from './chargeFlash';

const flip = (o: Partial<{ plugged: boolean; percent: number | null; secsLeft: number | null }> = {}) =>
  ({ plugged: false, percent: 50, secsLeft: null, ...o });

describe('flashTone', () => {
  it('is green on mains at any level', () => {
    expect(flashTone(true, 5)).toBe('#30d158');
    expect(flashTone(true, 100)).toBe('#30d158');
  });

  it('matches the Battery widget bands on battery', () => {
    expect(flashTone(false, 0)).toBe('#ff453a');
    expect(flashTone(false, 15)).toBe('#ff453a');
    expect(flashTone(false, 16)).toBe('#ff9f0a');
    expect(flashTone(false, 30)).toBe('#ff9f0a');
    expect(flashTone(false, 31)).toBe('#30d158');
    expect(flashTone(false, 100)).toBe('#30d158');
  });

  it('falls back to green with no battery', () => {
    expect(flashTone(false, null)).toBe('#30d158');
  });
});

describe('fillOf', () => {
  it('maps a percentage to 0..1 and clamps nonsense', () => {
    expect(fillOf(0)).toBe(0);
    expect(fillOf(50)).toBe(0.5);
    expect(fillOf(100)).toBe(1);
    expect(fillOf(140)).toBe(1);
    expect(fillOf(-5)).toBe(0);
    expect(fillOf(null)).toBe(1);
  });
});

describe('the flash schedule', () => {
  it('reads long enough to be read, and not so long it is in the way', () => {
    expect(FLASH_MS).toBe(2500);
    expect(FLASH_MS).toBeGreaterThan(1500);
    expect(FLASH_MS).toBeLessThan(4000);
  });

  it('enters within the 150-200 ms the design rules allow', () => {
    expect(FLASH.enter).toBeGreaterThanOrEqual(150);
    expect(FLASH.enter).toBeLessThanOrEqual(200);
  });

  it('holds still for longer than it spends moving', () => {
    expect(FLASH.hold).toBeGreaterThan(FLASH.enter + FLASH.exit);
  });
});

describe('flashText', () => {
  it('says charging when the cable goes in', () => {
    expect(flashText(flip({ plugged: true, percent: 42 }))).toEqual({ pct: '42%', label: 'Charging' });
  });

  it('says plugged in once it is full, not charging', () => {
    expect(flashText(flip({ plugged: true, percent: 97 })).label).toBe('Plugged in');
  });

  it('adds the time left only on battery', () => {
    expect(flashText(flip({ plugged: false, percent: 60, secsLeft: 8040 })).label).toBe('On battery · about 2h 14m left');
    expect(flashText(flip({ plugged: true, percent: 60, secsLeft: 8040 })).label).toBe('Charging');
  });

  it('leaves the time out when Windows does not know it', () => {
    expect(flashText(flip({ plugged: false, percent: 60, secsLeft: null })).label).toBe('On battery');
  });

  it('shows no fake number without a battery', () => {
    expect(flashText(flip({ percent: null })).pct).toBe('--');
  });
});
