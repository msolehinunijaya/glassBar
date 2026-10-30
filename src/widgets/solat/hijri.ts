// The Hijri date for the Waktu Solat panel: JAKIM's (or Aladhan's) when the prayer data has
// it, otherwise the Umm al-Qura calculation, marked an estimate. The Islamic day starts at
// Maghrib, so after Maghrib it is the next Hijri day. Tested in hijri.test.ts.
import type { PrayerDay } from '../../lib/ipc';

export const MONTHS_MS = ['Muharram', 'Safar', 'Rabiulawal', 'Rabiulakhir', 'Jamadilawal', 'Jamadilakhir',
  'Rejab', 'Syaaban', 'Ramadan', 'Syawal', 'Zulkaedah', 'Zulhijjah'];

export interface Hijri { y: number; m: number; d: number }

/** "1448-05-12" (JAKIM) or "12-05-1448" (Aladhan). */
export function parseHijri(s: string | null | undefined): Hijri | null {
  const a = s?.match(/^(\d{4})-(\d{1,2})-(\d{1,2})$/);
  if (a) return { y: +a[1], m: +a[2], d: +a[3] };
  const b = s?.match(/^(\d{1,2})-(\d{1,2})-(\d{4})$/);
  return b ? { y: +b[3], m: +b[2], d: +b[1] } : null;
}

export const hijriLabel = (h: Hijri) => `${h.d} ${MONTHS_MS[h.m - 1] ?? h.m} ${h.y} H`;

/** The Umm al-Qura date of a Gregorian day. */
export function hijriOf(date: Date): Hijri {
  const p = new Intl.DateTimeFormat('en-u-ca-islamic-umalqura', { year: 'numeric', month: 'numeric', day: 'numeric' }).formatToParts(date);
  const n = (t: string) => parseInt(p.find((x) => x.type === t)?.value ?? '0', 10);
  return { y: n('year'), m: n('month'), d: n('day') };
}

const ymd = (d: Date) => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;

/** Today's Hijri date as the panel shows it. */
export function hijriNow(days: PrayerDay[], now: number): { label: string; estimate: boolean; afterMaghrib: boolean; h: Hijri } {
  const d = new Date(now);
  const todayKey = ymd(d);
  const today = days.find((x) => x.date === todayKey);
  const maghrib = today?.times.find(([n]) => n === 'Maghrib')?.[1];
  const afterMaghrib = maghrib != null && now >= maghrib;
  const tomorrow = new Date(d.getFullYear(), d.getMonth(), d.getDate() + 1);
  const src = afterMaghrib ? days.find((x) => x.date === ymd(tomorrow)) : today;
  const known = parseHijri(src?.hijri);
  const h = known ?? hijriOf(afterMaghrib ? tomorrow : d);
  return { label: hijriLabel(h), estimate: !known, afterMaghrib, h };
}
