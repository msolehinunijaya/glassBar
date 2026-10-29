import { events, ipc, type IpuReading } from '../../lib/ipc';

/** Latest reading; Rust caches 15 min, this re-asks every 15 min.
 *  `key` = the location selection; when it changes, a fresh fetch is forced. */
export function liveIpu(key: () => string) {
  const s = $state<{ r: IpuReading | null; loading: boolean }>({ r: null, loading: true });
  let last = '';
  $effect(() => {
    const changed = key();
    const load = (force: boolean) => ipc.ipuGet(force).then((r) => { s.r = r; s.loading = false; });
    load(last !== '' && changed !== last);
    last = changed;
    const t = setInterval(() => load(false), 15 * 60_000);
    const off = events.changed('location', () => load(true));
    const off2 = events.refresh(() => load(true));
    return () => { clearInterval(t); off.then((f) => f()); off2.then((f) => f()); };
  });
  return s;
}

export const selectionKey = (s: { country: string; station: string; city: unknown }) => JSON.stringify([s.country, s.station, s.city]);
export const label = (r: IpuReading) => (r.kind === 'ipu' ? 'IPU' : 'AQI (US)');
