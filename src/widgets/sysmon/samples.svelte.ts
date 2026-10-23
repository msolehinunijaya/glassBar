import { events, ipc, type Sample } from '../../lib/ipc';

/** Last 60 samples (2 s apart). Rust keeps the history, so a new popup starts full. */
export function liveSamples() {
  const s = $state<{ list: Sample[] }>({ list: [] });
  $effect(() => {
    ipc.sysmonHistory().then((h) => { s.list = h; });
    const off = events.sysmonSample((x) => { s.list = [...s.list.slice(-59), x]; });
    return () => { off.then((f) => f()); };
  });
  return s;
}

/** 1234567 B/s -> "1.2 MB/s". */
export function rate(bps: number): string {
  const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
  let v = bps, i = 0;
  while (v >= 1000 && i < units.length - 1) { v /= 1000; i++; }
  return `${v < 10 && i ? v.toFixed(1) : Math.round(v)} ${units[i]}`;
}
