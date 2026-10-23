// Timer progress, computed from the stored end time (or the remaining time while paused),
// so it is right after sleep, pause or a restart and never drifts.
import type { Timer } from '../../lib/ipc';

export type TimerState = 'running' | 'paused' | 'warn' | 'done';

export interface Progress { state: TimerState; /** 0..1 of the duration elapsed */ elapsed: number; remainingMs: number }

/** Final stretch: the last 10% or the last minute, whichever is longer. */
export const warnFrom = (durationMs: number) => Math.max(60_000, durationMs * 0.1);

export function timerProgress(t: Timer, now: number): Progress {
  const total = Math.max(1, t.durationMs);
  if (t.endsAt === null && (t.remainingMs ?? 0) <= 0) return { state: 'done', elapsed: 1, remainingMs: 0 };
  const remaining = t.endsAt !== null ? Math.max(0, t.endsAt - now) : (t.remainingMs ?? 0);
  const elapsed = Math.min(1, Math.max(0, 1 - remaining / total));
  if (t.endsAt === null) return { state: 'paused', elapsed, remainingMs: remaining };
  if (remaining <= 0) return { state: 'done', elapsed: 1, remainingMs: 0 };
  return { state: remaining <= warnFrom(total) ? 'warn' : 'running', elapsed, remainingMs: remaining };
}

/** The timer the pill shows: a finished one first (until dismissed), else the one ending
 *  soonest among running and paused. `others` counts the rest that are still going. */
export function pickTimer(timers: Timer[], now: number): { timer: Timer | null; others: number } {
  const done = timers.find((t) => timerProgress(t, now).state === 'done');
  const active = timers.filter((t) => timerProgress(t, now).state !== 'done')
    .sort((a, b) => timerProgress(a, now).remainingMs - timerProgress(b, now).remainingMs);
  const timer = done ?? active[0] ?? null;
  return { timer, others: active.filter((t) => t !== timer).length };
}

/** Fill colour per state (finished: green). */
export const stateColor = (s: TimerState, accent = 'var(--accent)') =>
  s === 'warn' ? '#ff9f0a' : s === 'done' ? '#30d158' : accent;

/** Width of the fill: elapsed (fill up) or remaining (drain). */
export const fillFraction = (p: Progress, drain: boolean) => (drain ? 1 - p.elapsed : p.elapsed);
