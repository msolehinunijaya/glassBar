// The one place the widget reads privacy state: on open, whenever Rust says something changed
// (an app starts or stops using a resource, a switch flips, a device comes or goes, the mic
// mutes), after an action, and once a minute as a safety net (sleep/resume, the location
// service stopping, an unreported recorder starting, which Windows doesn't announce).
import { Camera, MapPin, Mic, MonitorDot, ScreenShare } from '@lucide/svelte';
import { events, ipc, type PrivacyKey, type PrivacyReport } from '../../lib/ipc';

export const ICON: Record<PrivacyKey, typeof Camera> = { camera: Camera, mic: Mic, location: MapPin, recording: MonitorDot, sharing: ScreenShare };

export function monitor(onChange?: (prev: PrivacyReport | null, next: PrivacyReport) => void) {
  const s = $state<{ v: PrivacyReport | null; error: string; busy: boolean; reload: () => Promise<void> }>({ v: null, error: '', busy: false, reload: async () => {} });
  $effect(() => {
    let alive = true;
    let seq = 0;
    const run = async () => {
      const mine = ++seq;
      s.busy = true;
      try {
        const r = await ipc.privacyReport();
        if (!alive || mine !== seq) return; // a newer read is on its way
        const prev = s.v;
        s.v = r;
        s.error = '';
        onChange?.(prev, r);
      } catch (e) {
        if (alive && mine === seq) s.error = String(e);
      } finally {
        if (mine === seq) s.busy = false;
      }
    };
    s.reload = run;
    run();
    const offs = [events.changed('privacy', run), events.changed('mic', run)];
    const t = setInterval(run, 60_000);
    return () => { alive = false; clearInterval(t); offs.forEach((o) => o.then((f) => f())); };
  });
  return s;
}
