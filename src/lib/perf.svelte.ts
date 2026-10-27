// The performance mode in force, in every window: this store for the few widgets that change
// in code, and <html data-perf="low|normal|high"> for the many that change in CSS. The bar
// also gets data-away while it is auto-hidden (animations pause then, except in High).
import { events, ipc, type PerfState } from './ipc';

export const perf = $state<PerfState>({ mode: 'normal', auto: false, reason: null });
/** The auto-hidden bar is slid away (bar window only). */
export const barAway = $state({ v: false });

export async function initPerf() {
  const root = document.documentElement;
  const load = () => ipc.perfState()
    .then((s) => { Object.assign(perf, s); root.dataset.perf = s.mode; })
    .catch(() => {});
  root.dataset.perf = perf.mode;
  await load();
  events.changed('perf', load).catch(() => {});
  if (root.dataset.window === 'bar') {
    events.barAway((away) => { barAway.v = away; if (away) root.dataset.away = ''; else delete root.dataset.away; }).catch(() => {});
  }
}
