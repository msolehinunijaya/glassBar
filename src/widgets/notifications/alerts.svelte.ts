import { events, ipc, type Alert } from '../../lib/ipc';

/** Live alert list, refreshed whenever Rust records or changes an alert. */
export function liveAlerts() {
  const s = $state<{ list: Alert[] }>({ list: [] });
  $effect(() => {
    const load = () => ipc.alertsList().then((l) => { s.list = l; });
    load();
    const off = events.alertsChanged(load);
    return () => { off.then((f) => f()); };
  });
  return s;
}
