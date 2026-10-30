import { describe, expect, it } from 'vitest';
import type { PomoState } from '../lib/ipc';
import { focusDock, focusShown } from './focusDock';

const MIN = 60_000;
const state = (pomo: Partial<PomoState['pomo']> = {}): PomoState => ({
  pomo: { phase: 'work', done: 1, endsAt: null, pausedMs: null, completed: null, ...pomo },
  cfg: { workMin: 25, shortMin: 5, longMin: 15, every: 4, autoStart: false, sound: true, volume: 60, goal: 4, dimOthers: false },
  label: '', today: { day: '2026-10-30', sessions: 1, minutes: 25, longest: 25 }, week: [], streak: 0, longest: 25,
});

describe('focus on the dock', () => {
  const now = 1_000_000_000;
  it('shows a running or paused focus session, not a break or an idle timer', () => {
    expect(focusDock(state({ endsAt: now + 10 * MIN }), now)).not.toBe(null);
    expect(focusDock(state({ pausedMs: 7 * MIN }), now)).not.toBe(null);
    expect(focusDock(state(), now)).toBe(null);
    expect(focusDock(state({ phase: 'short', endsAt: now + MIN }), now)).toBe(null);
    expect(focusDock(null, now)).toBe(null);
  });
  it('gives the time left, how far along, and when it started and ends', () => {
    const f = focusDock(state({ endsAt: now + (18 * 60 + 42) * 1000 }), now)!;
    expect(f).toMatchObject({ status: 'running', left: '18:42', session: 'Session 2 of 4', end: now + (18 * 60 + 42) * 1000 });
    expect(f.start).toBe(f.end! - 25 * MIN);
    expect(f.progress).toBeCloseTo((25 * 60 - (18 * 60 + 42)) / (25 * 60), 5);
    expect(f.dots).toEqual(['done', 'current', 'upcoming', 'upcoming']);
  });
  it('holds still while paused, with no end time', () => {
    const f = focusDock(state({ pausedMs: 5 * MIN }), now)!;
    expect(f).toMatchObject({ status: 'paused', left: '05:00', end: null, start: null });
    expect(f.progress).toBeCloseTo(0.8, 5);
  });
  it('steps aside while the pointer is on the dock, while editing, or for an alert', () => {
    expect(focusShown({ on: true, active: true, peek: false, edit: false, alert: false })).toBe(true);
    expect(focusShown({ on: false, active: true, peek: false, edit: false, alert: false })).toBe(false);
    expect(focusShown({ on: true, active: true, peek: true, edit: false, alert: false })).toBe(false);
    expect(focusShown({ on: true, active: true, peek: false, edit: true, alert: false })).toBe(false);
    expect(focusShown({ on: true, active: true, peek: false, edit: false, alert: true })).toBe(false);
    expect(focusShown({ on: true, active: false, peek: false, edit: false, alert: false })).toBe(false);
  });
});
