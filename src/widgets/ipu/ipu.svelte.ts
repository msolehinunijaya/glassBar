import { store } from '../../lib/config.svelte';
import { ipc, type IpuReading } from '../../lib/ipc';

/** Latest reading; Rust caches 15 min, this re-asks every 15 min. */
export function liveIpu(station: () => string) {
  const s = $state<{ r: IpuReading | null; loading: boolean }>({ r: null, loading: true });
  let last = ''; // force a fresh fetch when the station or location changes
  $effect(() => {
    const changed = station() + JSON.stringify(store.config?.location);
    const load = (force: boolean) => ipc.ipuGet(force).then((r) => { s.r = r; s.loading = false; });
    load(changed !== last);
    last = changed;
    const t = setInterval(() => load(false), 15 * 60_000);
    return () => clearInterval(t);
  });
  return s;
}

export const label = (r: IpuReading) => (r.kind === 'ipu' ? 'IPU' : 'AQI (US)');
