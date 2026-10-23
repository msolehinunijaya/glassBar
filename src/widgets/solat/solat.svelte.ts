import { events, ipc, type SolatInfo } from '../../lib/ipc';
import { nextPrayer } from './next';
import type { SolatSettings } from './index';

/** Prayer data (Rust caches it per day) plus a clock that ticks every minute,
 *  or every second during the last 10 minutes before a prayer. */
export function liveSolat(settings: () => SolatSettings) {
  const s = $state<{ info: SolatInfo | null; error: string; now: number }>({ info: null, error: '', now: Date.now() });

  $effect(() => {
    settings().zone; // refetch on zone change (and below, when the shared location moves)
    const load = () => ipc.solatDays().then((i) => { s.info = i; s.error = ''; }).catch((e) => { s.error = String(e); });
    load();
    const t = setInterval(load, 30 * 60_000);
    const off = events.changed('location', load);
    return () => { clearInterval(t); off.then((f) => f()); };
  });

  const next = $derived(s.info ? nextPrayer(s.info.days, s.now, settings().showImsak) : null);

  $effect(() => {
    const left = next ? next.at - s.now : Infinity;
    // Every second in the last 15 min and just after a prayer (flashing state); else on the minute.
    const sincePrev = s.info ? Math.min(...s.info.days.flatMap((d) => d.times.map(([, t]) => (s.now >= t ? s.now - t : Infinity)))) : Infinity;
    const wait = left <= 15 * 60_000 || sincePrev <= 31 * 60_000 ? 1000 : 60_000 - (s.now % 60_000);
    const t = setTimeout(() => { s.now = Date.now(); }, Math.max(250, wait));
    return () => clearTimeout(t);
  });

  return {
    get info() { return s.info; },
    get error() { return s.error; },
    get now() { return s.now; },
    get next() { return next; },
  };
}
