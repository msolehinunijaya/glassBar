import { ipc, type DiskInfo } from '../../lib/ipc';

/** Drive list, refreshed every minute while a pill or panel is mounted (and enabled). */
export function liveDrives(every = 60_000, enabled: () => boolean = () => true) {
  const s = $state({ list: [] as DiskInfo[], loaded: false });
  $effect(() => {
    if (!enabled()) return; // e.g. Disk not ticked: don't poll at all
    const load = () => ipc.disksList().then((l) => { s.list = l; s.loaded = true; }).catch(() => (s.loaded = true));
    load();
    const t = setInterval(load, every);
    return () => clearInterval(t);
  });
  return s;
}
