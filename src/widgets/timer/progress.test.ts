import { describe, expect, it } from 'vitest';
import type { Timer } from '../../lib/ipc';
import { fillFraction, pickTimer, stateColor, timerProgress } from './progress';

const MIN = 60_000;
const T = (id: number, durationMs: number, endsAt: number | null, remainingMs: number | null = null): Timer => ({ id, label: '', durationMs, endsAt, remainingMs });

describe('timerProgress', () => {
  it('measures from the real end time', () => {
    const p = timerProgress(T(1, 10 * MIN, 1_000 + 6 * MIN), 1_000);
    expect(p.elapsed).toBeCloseTo(0.4);
    expect(p.state).toBe('running');
  });
  it('is right after sleep: the end time, not ticks, decides', () => {
    // started at 0 for 10 min; the PC slept and wakes at 9.5 min
    const p = timerProgress(T(1, 10 * MIN, 10 * MIN), 9.5 * MIN);
    expect(p.elapsed).toBeCloseTo(0.95);
    expect(p.state).toBe('warn');
  });
  it('paused keeps its progress; resume continues from it', () => {
    const paused = timerProgress(T(1, 10 * MIN, null, 4 * MIN), 99 * MIN);
    expect(paused).toEqual({ state: 'paused', elapsed: 0.6, remainingMs: 4 * MIN });
    const resumed = timerProgress(T(1, 10 * MIN, 50 * MIN + 4 * MIN), 50 * MIN);
    expect(resumed.elapsed).toBeCloseTo(0.6);
  });
  it('warns in the final minute (or last 10% of long timers)', () => {
    expect(timerProgress(T(1, 5 * MIN, 61_000), 2_000).state).toBe('warn');
    expect(timerProgress(T(1, 60 * MIN, 7 * MIN), 0).state).toBe('running');
    expect(timerProgress(T(1, 60 * MIN, 5 * MIN), 0).state).toBe('warn');
  });
  it('finished: ended, or stored as done (remaining 0)', () => {
    expect(timerProgress(T(1, MIN, 1000), 5000).state).toBe('done');
    expect(timerProgress(T(1, MIN, null, 0), 5000)).toEqual({ state: 'done', elapsed: 1, remainingMs: 0 });
  });
});

describe('pickTimer', () => {
  it('shows the one ending soonest and counts the rest', () => {
    const r = pickTimer([T(1, 10 * MIN, 9 * MIN), T(2, 10 * MIN, 3 * MIN), T(3, 10 * MIN, null, 5 * MIN)], 0);
    expect(r.timer?.id).toBe(2);
    expect(r.others).toBe(2);
  });
  it('a finished timer comes first until dismissed', () => {
    const r = pickTimer([T(1, MIN, 9 * MIN), T(2, MIN, null, 0)], 0);
    expect(r.timer?.id).toBe(2);
    expect(r.others).toBe(1);
  });
  it('nothing set', () => expect(pickTimer([], 0)).toEqual({ timer: null, others: 0 }));
});

describe('state colours and direction', () => {
  it('maps states to colours', () => {
    expect(stateColor('running', '#0a84ff')).toBe('#0a84ff');
    expect(stateColor('paused', '#0a84ff')).toBe('#0a84ff');
    expect(stateColor('warn')).toBe('#ff9f0a');
    expect(stateColor('done')).toBe('#30d158');
  });
  it('fills up or drains', () => {
    const p = { state: 'running' as const, elapsed: 0.25, remainingMs: 1 };
    expect(fillFraction(p, false)).toBe(0.25);
    expect(fillFraction(p, true)).toBe(0.75);
  });
});
