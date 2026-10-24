import type { PrayerDay } from '../../lib/ipc';

/** Next prayer across today and tomorrow (Syuruk never; Imsak only if shown). */
export function nextPrayer(days: PrayerDay[], now: number, imsak: boolean): { name: string; at: number } | null {
  for (const d of days) {
    for (const [name, at] of d.times) {
      if (name === 'Syuruk' || (!imsak && name === 'Imsak')) continue;
      if (at > now) return { name, at };
    }
  }
  return null;
}

/** "in 2h 05m" / "in 08:12" (last 10 minutes show seconds). */
export function countdown(ms: number): string {
  const s = Math.max(0, Math.floor(ms / 1000));
  if (s < 600) return `${String(Math.floor(s / 60)).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`;
  const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60);
  return h ? `${h}h ${String(m).padStart(2, '0')}m` : `${m}m`;
}
