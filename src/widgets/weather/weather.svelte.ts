import { events, ipc, type Weather } from '../../lib/ipc';
import { store } from '../../lib/config.svelte';

/** Weather that refreshes every 30 min (Rust caches; this only asks). */
export function liveWeather() {
  const s = $state<{ w: Weather | null; loading: boolean }>({ w: null, loading: true });
  $effect(() => {
    // Re-fetch when the unit changes, or the shared location really moves.
    // (Units live in the clock's settings since weather moved onto the clock.)
    Object.values(store.config?.profiles ?? {}).flatMap((p) => p.widgets.filter((w) => w.id === 'clock').map((w) => w.settings?.weatherUnit)).join();
    const load = (force = false) => ipc.weatherGet(force).then((w) => { s.w = w; s.loading = false; });
    load();
    const t = setInterval(load, 30 * 60_000);
    const off = events.changed('location', () => load());
    return () => { clearInterval(t); off.then((f) => f()); };
  });
  return s;
}

export const deg = (t: number) => `${Math.round(t)}°`;
