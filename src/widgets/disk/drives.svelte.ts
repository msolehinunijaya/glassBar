import { ipc, type DiskInfo } from '../../lib/ipc';

/** Drive list, refreshed every minute while a pill or panel is mounted. */
export function liveDrives(every = 60_000) {
  const s = $state({ list: [] as DiskInfo[], loaded: false });
  $effect(() => {
    const load = () => ipc.disksList().then((l) => { s.list = l; s.loaded = true; }).catch(() => (s.loaded = true));
    load();
    const t = setInterval(load, every);
    return () => clearInterval(t);
  });
  return s;
}
