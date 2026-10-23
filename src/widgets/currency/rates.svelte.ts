import { ipc, type Rates } from '../../lib/ipc';

/** Rates for `base()`, re-asked every 30 min (Rust caches for an hour). */
export function liveRates(base: () => string) {
  const s = $state<{ r: Rates | null; error: string; loading: boolean }>({ r: null, error: '', loading: true });
  const load = (force = false) => {
    const b = base();
    s.loading = true;
    return ipc.fxRates(b, force)
      .then((r) => { s.r = r; s.error = ''; })
      .catch((e) => { s.error = String(e); if (s.r?.base !== b) s.r = null; })
      .finally(() => (s.loading = false));
  };
  $effect(() => {
    base();
    load();
    const t = setInterval(load, 30 * 60_000);
    return () => clearInterval(t);
  });
  return Object.assign(s, { refresh: () => load(true) });
}
