// Pomodoro: a part of the Productivity widget (pomodoro · to-do · notes).
import type { Phase, PomoCfg } from '../../lib/ipc';

/** What the timer is doing, in the words the UI shows. Phases never start on their own. */
export function stateOf(s: { phase: Phase; endsAt: number | null; pausedMs: number | null; completed: Phase | null }) {
  const startLabel = s.phase === 'work' ? 'Start Work Session' : s.phase === 'long' ? 'Start Long Break' : 'Start Break';
  if (s.endsAt) return { key: 'running' as const, title: s.phase === 'work' ? 'Working' : 'On Break', action: 'Pause' };
  if (s.pausedMs != null) return { key: 'paused' as const, title: 'Paused', action: 'Resume' };
  if (s.completed) return { key: 'completed' as const, title: s.completed === 'work' ? 'Work Session Complete' : 'Break Complete', action: startLabel };
  return { key: 'ready' as const, title: s.phase === 'work' ? 'Ready to focus' : s.phase === 'long' ? 'Long Break Ready' : 'Break Ready', action: startLabel };
}

export const PHASE = {
  work: { name: 'Focus', color: '#ff6b5b' },
  short: { name: 'Short break', color: '#30d158' },
  long: { name: 'Long break', color: '#64d2ff' },
} as const;

