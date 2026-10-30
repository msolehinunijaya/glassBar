import { describe, expect, it } from 'vitest';
import { awakeText, levelTone, micDock, usingText } from './extras';

describe('Keep awake text', () => {
  const now = 1_000_000;
  it('says off, until off, or the time left', () => {
    expect(awakeText({ on: false, until: null, screen: true }, now)).toBe('Off');
    expect(awakeText({ on: true, until: null, screen: true }, now)).toBe('On · until you turn it off');
    expect(awakeText({ on: true, until: now + (84 * 60 + 5) * 1000, screen: true }, now)).toBe('On · 1:25 left');
    expect(awakeText({ on: true, until: now + 42 * 60_000, screen: true }, now)).toBe('On · 42m left');
    expect(awakeText({ on: true, until: now + 20_000, screen: true }, now)).toBe('On · 1m left');
  });
});

describe('Mic on the dock', () => {
  it('shows only news in automatic mode', () => {
    expect(micDock('auto', false, false)).toBe(null);
    expect(micDock('auto', true, false)).toBe('muted');
    expect(micDock('auto', false, true)).toBe('live');
    expect(micDock('auto', true, true)).toBe('muted');
    expect(micDock('never', true, true)).toBe(null);
    expect(micDock('always', false, false)).toBe('idle');
  });
  it('names who uses it', () => {
    expect(usingText([])).toBe('');
    expect(usingText([{ name: 'Zoom', exe: '', since: 0 }])).toBe('Zoom');
    expect(usingText([{ name: 'Zoom', exe: '', since: 0 }, { name: 'Teams', exe: '', since: 0 }, { name: 'OBS', exe: '', since: 0 }])).toBe('Zoom +2');
  });
});

describe('Battery tone', () => {
  it('is amber at 20 % and red at 10 %', () => {
    expect(levelTone(50)).toBe('ok');
    expect(levelTone(20)).toBe('low');
    expect(levelTone(10)).toBe('critical');
  });
});
