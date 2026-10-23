import { store } from '../../lib/config.svelte';
import { ipc, type IpuReading } from '../../lib/ipc';

/** Latest reading; Rust caches 15 min, this re-asks every 15 min.
 *  `key` = the location selection; when it changes, a fresh fetch is forced. */
export function liveIpu(key: () => string) {
  const s = $state<{ r: IpuReading | null; loading: boolean }>({ r: null, loading: true });
  let last = '';
  $effect(() => {
    const changed = key() + JSON.stringify(store.config?.location);
    const load = (force: boolean) => ipc.ipuGet(force).then((r) => { s.r = r; s.loading = false; });
    load(last !== '' && changed !== last);
    last = changed;
    const t = setInterval(() => load(false), 15 * 60_000);
    return () => clearInterval(t);
  });
  return s;
}

export const selectionKey = (s: { country: string; station: string; city: unknown }) => JSON.stringify([s.country, s.station, s.city]);
export const label = (r: IpuReading) => (r.kind === 'ipu' ? 'IPU' : 'AQI (US)');
