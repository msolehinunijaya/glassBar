import { ipc, type SolatInfo } from '../../lib/ipc';
import { store } from '../../lib/config.svelte';
import { nextPrayer } from './next';
import type { SolatSettings } from './index';

/** Prayer data (Rust caches it per day) plus a clock that ticks every minute,
 *  or every second during the last 10 minutes before a prayer. */
export function liveSolat(settings: () => SolatSettings) {
  const s = $state<{ info: SolatInfo | null; error: string; now: number }>({ info: null, error: '', now: Date.now() });

  $effect(() => {
    settings().zone; JSON.stringify(store.config?.location); // refetch on zone/location change
    const load = () => ipc.solatDays().then((i) => { s.info = i; s.error = ''; }).catch((e) => { s.error = String(e); });
    load();
    const t = setInterval(load, 30 * 60_000);
    return () => clearInterval(t);
  });

  const next = $derived(s.info ? nextPrayer(s.info.days, s.now, settings().showImsak) : null);

  $effect(() => {
    const left = next ? next.at - s.now : Infinity;
    const wait = left <= 600_000 ? 1000 : 60_000 - (s.now % 60_000);
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
