import { ipc, type PomoState } from '../../lib/ipc';
import { live } from '../../lib/live.svelte';
import { tickingNow } from '../../lib/now.svelte';
import { view } from './index';

/** Live Pomodoro state, its view, and the time left (ticking while it runs). */
export function livePomo() {
  const s = live<PomoState | null>('pomodoro', ipc.pomoGet, null);
  const running = () => !!s.v?.pomo.endsAt;
  const t = tickingNow(running);
  const v = $derived(s.v ? view(s.v) : null);
  return {
    get state() { return s.v; },
    get view() { return v; },
    get running() { return running(); },
    get remaining() {
      const p = s.v?.pomo;
      if (!p || !s.v) return 0;
      if (p.endsAt) return Math.max(0, p.endsAt - t.now);
      if (p.pausedMs != null) return p.pausedMs;
      return phaseMs(s.v);
    },
    /** Time left as a fraction of the phase (1 = full). */
    get left() { return s.v ? Math.min(1, this.remaining / phaseMs(s.v)) : 1; },
    get elapsed() { return s.v ? phaseMs(s.v) - this.remaining : 0; },
  };
}

function phaseMs(s: PomoState) {
  const m = s.pomo.phase === 'work' ? s.cfg.workMin : s.pomo.phase === 'short' ? s.cfg.shortMin : s.cfg.longMin;
  return Math.max(1, m) * 60_000;
}

/** Start or pause (the primary action, the dock's ⏯ and Space). */
export const toggle = (running: boolean) => (running ? ipc.pomoPause() : ipc.pomoStart());
