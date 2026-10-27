import { events, ipc, type WaterToday } from '../../lib/ipc';

/** Today's water, kept current (Rust sends `water:changed` after every add or remove). */
export function liveWater() {
  let v = $state<WaterToday | null>(null);
  $effect(() => {
    ipc.waterToday().then((t) => (v = t)).catch(() => {});
    const off = events.waterChanged((t) => (v = t));
    // A new day starts at midnight: ask again then.
    const now = new Date();
    const midnight = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1).getTime() - now.getTime();
    const t = setTimeout(() => ipc.waterToday().then((x) => (v = x)).catch(() => {}), midnight + 1000);
    return () => { off.then((f) => f()); clearTimeout(t); };
  });
  return { get v() { return v; } };
}
