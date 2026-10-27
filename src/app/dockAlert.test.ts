import { describe, expect, it } from 'vitest';
import { arrive, holdMs, viewOf } from './dockAlert';

describe('arrive', () => {
  it('shows anything on an empty dock', () => {
    expect(arrive(null, 'volume')).toBe('show');
  });
  it('updates the same kind in place', () => {
    expect(arrive('volume', 'volume')).toBe('update');
    expect(arrive('charger', 'charger')).toBe('update');
  });
  it('lets a higher alert replace a lower one', () => {
    expect(arrive('volume', 'charger')).toBe('show');
    expect(arrive('charger', 'prayer')).toBe('show');
  });
  it('drops a lower alert instead of queueing it', () => {
    expect(arrive('prayer', 'volume')).toBe('drop');
    expect(arrive('charger', 'brightness')).toBe('drop');
  });
  it('volume and brightness replace each other', () => {
    expect(arrive('volume', 'brightness')).toBe('show');
    expect(arrive('brightness', 'volume')).toBe('show');
  });
});

describe('holdMs', () => {
  it('keeps the charger flash timing at the normal length', () => {
    expect(holdMs('charger', 'normal')).toBe(2100);
  });
  it('waits longest for the prayer alert, which has a button', () => {
    expect(holdMs('prayer', 'normal')).toBeGreaterThan(holdMs('charger', 'normal'));
  });
  it('scales with the length setting', () => {
    expect(holdMs('volume', 'short')).toBeLessThan(holdMs('volume', 'normal'));
    expect(holdMs('volume', 'long')).toBeGreaterThan(holdMs('volume', 'normal'));
  });
});

describe('viewOf', () => {
  it('reads the charger like the old flash did', () => {
    const v = viewOf({ kind: 'charger', plugged: true, percent: 40, secsLeft: null });
    expect(v).toMatchObject({ icon: 'plug', value: '40%', label: 'Charging', fill: 0.4 });
  });
  it('shows volume as a level, and Muted with an empty fill', () => {
    expect(viewOf({ kind: 'volume', percent: 48, muted: false })).toMatchObject({ icon: 'volume', value: '48%', fill: 0.48 });
    expect(viewOf({ kind: 'volume', percent: 48, muted: true })).toMatchObject({ icon: 'mute', value: 'Muted', fill: 0 });
  });
  it('clamps nonsense levels', () => {
    expect(viewOf({ kind: 'brightness', percent: 140 }).value).toBe('100%');
  });
  it('names the prayer and its time, with no fill', () => {
    const v = viewOf({ kind: 'prayer', name: 'Zohor', at: 0 }, () => '1:15 PM');
    expect(v).toMatchObject({ icon: 'prayer', value: 'Zohor', label: '1:15 PM · prayer time', fill: null });
  });
});

describe('the water alert', () => {
  it('sits between prayer time and the charger', () => {
    expect(arrive('charger', 'water')).toBe('show');
    expect(arrive('water', 'charger')).toBe('drop');
    expect(arrive('water', 'prayer')).toBe('show');
  });
  it('shows today so far against the goal', () => {
    expect(viewOf({ kind: 'water', total: 750, goal: 2000, cup: 250 })).toMatchObject({ icon: 'water', value: '750 ml', label: 'Time to drink · goal 2,000 ml', fill: 0.375 });
    expect(viewOf({ kind: 'water', total: 2000, goal: 2000, cup: 250, reached: true }).label).toBe('Goal of 2,000 ml reached today');
  });
});

describe('the Pomodoro alert', () => {
  it('says which phase ended, in its colour', () => {
    expect(viewOf({ kind: 'pomodoro', done: 'focus', body: 'Your break has started.' })).toMatchObject({ icon: 'break', value: 'Focus done', label: 'Your break has started.', tone: '#30d158' });
    expect(viewOf({ kind: 'pomodoro', done: 'break', body: 'Focus has started.' })).toMatchObject({ icon: 'focus', value: 'Break over', tone: '#ff9f0a' });
  });
  it('outranks the charger and volume, not prayer time', () => {
    expect(arrive('charger', 'pomodoro')).toBe('show');
    expect(arrive('pomodoro', 'volume')).toBe('drop');
    expect(arrive('pomodoro', 'prayer')).toBe('show');
  });
});

describe('the timer alert', () => {
  it("says time's up with the timer's name", () => {
    expect(viewOf({ kind: 'timer', id: 3, label: 'Tea is ready' })).toMatchObject({ icon: 'timer', value: "Time's up", label: 'Tea is ready' });
    expect(viewOf({ kind: 'timer', id: 3, label: '' }).label).toBe('Timer done');
  });
});
