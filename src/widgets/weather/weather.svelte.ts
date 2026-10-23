import { ipc, type Weather } from '../../lib/ipc';
import { store } from '../../lib/config.svelte';

/** Weather that refreshes every 30 min (Rust caches; this only asks). */
export function liveWeather() {
  const s = $state<{ w: Weather | null; loading: boolean }>({ w: null, loading: true });
  $effect(() => {
    // Re-fetch when the unit or location settings change.
    JSON.stringify(store.config?.location);
    // (Units live in the clock's settings since weather moved onto the clock.)
    Object.values(store.config?.profiles ?? {}).flatMap((p) => p.widgets.filter((w) => w.id === 'clock').map((w) => w.settings?.weatherUnit)).join();
    const load = (force = false) => ipc.weatherGet(force).then((w) => { s.w = w; s.loading = false; });
    load();
    const t = setInterval(load, 30 * 60_000);
    return () => clearInterval(t);
  });
  return s;
}

export const deg = (t: number) => `${Math.round(t)}°`;
