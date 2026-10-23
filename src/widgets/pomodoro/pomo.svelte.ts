import { ipc, type PomoState } from '../../lib/ipc';
import { live } from '../../lib/live.svelte';
import { tickingNow } from '../../lib/now.svelte';

/** Live Pomodoro state plus derived remaining time and progress. */
export function livePomo() {
  const s = live<PomoState | null>('pomodoro', ipc.pomoGet, null);
  const running = () => !!s.v?.pomo.endsAt;
  const t = tickingNow(running);
  return {
    get state() { return s.v; },
    get running() { return running(); },
    get remaining() {
      const p = s.v?.pomo;
      if (!p || !s.v) return 0;
      if (p.endsAt) return Math.max(0, p.endsAt - t.now);
      if (p.pausedMs != null) return p.pausedMs;
      return phaseMs(s.v);
    },
    get progress() { return s.v ? 1 - this.remaining / phaseMs(s.v) : 0; },
  };
}

function phaseMs(s: PomoState) {
  const m = s.pomo.phase === 'work' ? s.cfg.workMin : s.pomo.phase === 'short' ? s.cfg.shortMin : s.cfg.longMin;
  return Math.max(1, m) * 60_000;
}
