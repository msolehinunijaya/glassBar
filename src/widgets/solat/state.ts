import type { PrayerDay } from '../../lib/ipc';

export interface Prayer { name: string; at: number; key: string }
export interface PrayerState {
  /** countdown → approaching (last 15 min) → reached (time came, not yet ✓'d) */
  phase: 'countdown' | 'approaching' | 'reached';
  next: Prayer;
  /** The prayer whose time has come and is waiting for ✓. */
  reached: Prayer | null;
  /** Time elapsed from the previous prayer to the next, 0..1. */
  progress: number;
}

const APPROACH_MS = 15 * 60_000;
const DAY_MS = 86_400_000;
/** A prayer only flashes if it arrived within this window (no flashing for one that
 *  passed hours before GlassBar started). */
const REACHED_WINDOW_MS = 30 * 60_000;

export function prayerState(days: PrayerDay[], now: number, ackKey: string | null, imsak: boolean): PrayerState | null {
  const all: Prayer[] = days.flatMap((d) => d.times
    .filter(([n]) => n !== 'Syuruk' && (imsak || n !== 'Imsak'))
    .map(([name, at]) => ({ name, at, key: `${d.date}:${name}` })))
    .sort((a, b) => a.at - b.at);
  const next = all.find((p) => p.at > now);
  if (!next) return null;
  const prev = [...all].reverse().find((p) => p.at <= now);
  const reached = prev && now - prev.at <= REACHED_WINDOW_MS && prev.key !== ackKey ? prev : null;
  // Between midnight and Subuh the previous prayer is last night's Isyak, which isn't loaded
  // (only today and tomorrow are). Today's last prayer a day earlier is within a minute of it,
  // and only drives the progress fill (never the "reached" alert).
  const nextDate = next.key.slice(0, next.key.indexOf(':'));
  const lastToday = all.filter((p) => p.key.startsWith(nextDate + ':')).at(-1);
  const start = prev?.at ?? (lastToday ? lastToday.at - DAY_MS : null);
  const progress = start != null && start < now ? Math.min(1, (now - start) / (next.at - start)) : 0;
  const phase = reached ? 'reached' : next.at - now <= APPROACH_MS ? 'approaching' : 'countdown';
  return { phase, next, reached, progress };
}
