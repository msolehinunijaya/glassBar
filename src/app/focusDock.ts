// Focus on the dock: while a Pomodoro focus session runs (or is paused), the whole dock shows
// it: time left, a timeline from start to end, and the controls. Hovering the dock brings
// the widgets back; leaving it returns to the focus view. Pure; tested in focusDock.test.ts.
import type { PomoState } from '../lib/ipc';
import { clock } from '../lib/time';
import { view, type Dot } from '../widgets/pomodoro/index';

export interface FocusDock {
  status: 'running' | 'paused';
  /** "18:42" (or "1:05:09"). */
  left: string;
  /** Elapsed share of the session, 0..1. */
  progress: number;
  /** When the session started and ends (Unix ms); null while paused. */
  start: number | null;
  end: number | null;
  session: string;
  dots: Dot[];
}

/** The focus view for this state, or null when there is no focus session to show. */
export function focusDock(s: PomoState | null, now: number): FocusDock | null {
  if (!s || s.pomo.phase !== 'work') return null;
  const { endsAt, pausedMs } = s.pomo;
  if (endsAt == null && pausedMs == null) return null;
  const total = Math.max(1, s.cfg.workMin) * 60_000;
  const remaining = Math.max(0, endsAt != null ? endsAt - now : pausedMs!);
  const v = view(s);
  return {
    status: endsAt != null ? 'running' : 'paused',
    left: clock(remaining),
    progress: Math.min(1, Math.max(0, 1 - remaining / total)),
    start: endsAt != null ? endsAt - total : null,
    end: endsAt ?? null,
    session: v.session.split(' · ')[0],
    dots: v.dots,
  };
}

/** Whether the focus view covers the dock now. `on`: the setting; `active`: a session to show;
 *  `peek`: the pointer is on the dock (widgets back); `alert`: a dock alert has the dock. */
export const focusShown = (o: { on: boolean; active: boolean; peek: boolean; edit: boolean; alert: boolean }) =>
  o.on && o.active && !o.peek && !o.edit && !o.alert;
