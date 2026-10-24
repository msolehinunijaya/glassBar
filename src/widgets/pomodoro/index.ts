// Pomodoro: a part of the Productivity widget (pomodoro · to-do · notes).
// `view()` turns one PomoState into everything the pill and the flyout show, so the session
// dots, the session text and Today always come from the same read and never disagree.
import type { Phase, PomoState } from '../../lib/ipc';
import { fmtMinutes } from '../../lib/time';

/** The phase colour is a theme token (styles/glass.css): one loud colour at a time. */
export const PHASE: Record<Phase, { name: string; color: string; key: string }> = {
  work: { name: 'Focus', color: 'var(--phase-focus)', key: '1' },
  short: { name: 'Short break', color: 'var(--phase-short)', key: '2' },
  long: { name: 'Long break', color: 'var(--phase-long)', key: '3' },
};
export const PHASES: Phase[] = ['work', 'short', 'long'];

export type Dot = 'done' | 'current' | 'upcoming';
export type Status = 'running' | 'paused' | 'ready';

export function view(s: PomoState) {
  const { phase, done: rawDone, endsAt, pausedMs, completed } = s.pomo;
  const every = Math.max(1, s.cfg.every);
  const done = Math.min(rawDone, every);
  const status: Status = endsAt ? 'running' : pausedMs != null ? 'paused' : 'ready';
  const n = Math.min(done + 1, every); // the focus session in progress (or next)
  const label = status === 'paused' ? 'Paused' : status === 'ready' ? 'Ready' : phase === 'work' ? 'Focusing' : PHASE[phase].name;
  const primary = status === 'running' ? 'Pause' : status === 'paused' ? 'Resume' : phase === 'work' ? 'Start focus' : 'Start break';
  const dots: Dot[] = Array.from({ length: every }, (_, i) => (i < done ? 'done' : phase === 'work' && i === done ? 'current' : 'upcoming'));
  const session = phase === 'work'
    ? `Session ${n} of ${every} · then ${n >= every ? 'long' : 'short'} break`
    : phase === 'short' ? `${done} of ${every} done · next: session ${n}`
    : `${done} of ${every} done · then a new round`;
  const goal = Math.max(1, s.cfg.goal || 4);
  const { minutes, sessions } = s.today;
  return {
    phase, status, label, primary, dots, session,
    /** "3/4": the focus session now (on a break: sessions done). */
    count: `${phase === 'work' ? n : done}/${every}`,
    skip: phase === 'work' ? 'Skip to break' : 'Skip to focus',
    today: { minutes, sessions, goal, pct: Math.min(1, sessions / goal), text: `${fmtMinutes(minutes)} · ${sessions} ${sessions === 1 ? 'session' : 'sessions'}` },
    /** Never started today: the dock shows a plain tomato, no timer. */
    idle: status === 'ready' && !completed && sessions === 0 && rawDone === 0 && phase === 'work',
  };
}
export type PomoView = ReturnType<typeof view>;
