// The shared location, one copy per window: loaded once, then kept current by Rust's
// `location:status` events. Widgets reload their data on `location:changed` instead.
import { events, ipc, type LocationStatus } from './ipc';

export const location = $state<{ s: LocationStatus | null; busy: boolean }>({ s: null, busy: false });
let started = false;

export function useLocation() {
  if (!started) {
    started = true;
    events.location((s) => (location.s = s));
    ipc.locationStatus().then((s) => (location.s = s)).catch(() => {});
  }
  return location;
}

export async function refreshLocation() {
  location.busy = true;
  try { location.s = await ipc.locationStatus(true); } finally { location.busy = false; }
}
