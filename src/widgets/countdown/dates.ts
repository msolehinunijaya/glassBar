// Countdown dates: next occurrences in the Gregorian, Hijri (Umm al-Qura) and Chinese
// calendars, and "how long until" in words. Pure; tested in dates.test.ts.

/** repeat: none · yearly (same day every year) · hijri / chinese (that calendar's month and
 *  day, `rule`). `date` is the next occurrence (YYYY-MM-DD), kept current by rollOver. */
export interface Countdown {
  id: string; name: string; date: string; time: string | null; emoji: string;
  repeat: 'none' | 'yearly' | 'hijri' | 'chinese'; rule: { m: number; d: number } | null;
  created: number; remindDayBefore: boolean;
}

const DAY = 86_400_000;
export const ymd = (d: Date) => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
export const parseYmd = (s: string) => { const [y, m, d] = s.split('-').map(Number); return new Date(y, m - 1, d); };
const startOfDay = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate());
const addDays = (d: Date, n: number) => new Date(d.getFullYear(), d.getMonth(), d.getDate() + n);

/** Month and day of `d` in a calendar Intl knows ("islamic-umalqura", "chinese"). */
export function calParts(cal: string, d: Date): { m: number; d: number } {
  const p = new Intl.DateTimeFormat(`en-u-ca-${cal}`, { month: 'numeric', day: 'numeric' }).formatToParts(d);
  return { m: parseInt(p.find((x) => x.type === 'month')?.value ?? '0', 10), d: parseInt(p.find((x) => x.type === 'day')?.value ?? '0', 10) };
}

/** The first day on or after `from` that is `month`/`day` in `cal` (searched over 400 days).
 *  A 30th that this year's month lacks falls back to the month's last day. */
export function nextCal(cal: string, month: number, day: number, from: Date): Date {
  let d = startOfDay(from);
  let lastOfMonth: Date | null = null;
  for (let i = 0; i < 400; i++, d = addDays(d, 1)) {
    const p = calParts(cal, d);
    if (p.m === month && p.d === day) return d;
    if (p.m === month) lastOfMonth = d;
    else if (lastOfMonth) return lastOfMonth;
  }
  return startOfDay(from);
}

/** The next "MM-DD" on or after `from` (today counts). */
export function nextYearly(mmdd: string, from: Date): Date {
  const [m, d] = mmdd.split('-').map(Number);
  const y = from.getFullYear();
  const t = new Date(y, m - 1, d);
  return t >= startOfDay(from) ? t : new Date(y + 1, m - 1, d);
}

/** When the countdown ends: its date at its time, or the start of that day. */
export function occurrence(c: Countdown, _now: Date): Date {
  const d = parseYmd(c.date);
  if (c.time) { const [h, m] = c.time.split(':').map(Number); d.setHours(h, m); }
  return d;
}

/** A repeating countdown whose day is over moves to its next occurrence; others stay. */
export function rollOver(c: Countdown, now: Date): Countdown {
  if (c.repeat === 'none' || addDays(parseYmd(c.date), 1) > now) return c;
  const from = startOfDay(now);
  const next = c.repeat === 'yearly' ? nextYearly(c.date.slice(5), from)
    : c.rule ? nextCal(c.repeat === 'hijri' ? 'islamic-umalqura' : 'chinese', c.rule.m, c.rule.d, from) : null;
  return next ? { ...c, date: ymd(next) } : c;
}

export interface Left { big: string; small: string; short: string; state: 'future' | 'today' | 'done'; ms: number }
const plural = (n: number, w: string) => `${n} ${w}${n === 1 ? '' : 's'}`;

/** How long until `target`. `timed`: the countdown has a time of day (hours and minutes in
 *  the last day); otherwise whole calendar days, "Tomorrow", "Today". */
export function left(target: Date, timed: boolean, now: Date): Left {
  const ms = target.getTime() - now.getTime();
  const days = Math.round((startOfDay(target).getTime() - startOfDay(now).getTime()) / DAY);
  if (days < 0 || (timed && ms < 0 && days < 0)) return { big: 'Done', small: '', short: 'Done', state: 'done', ms };
  if (timed && ms >= 0 && ms < DAY) {
    const m = Math.max(1, Math.ceil(ms / 60_000));
    return m < 60
      ? { big: `${m}m`, small: '', short: `${m}m`, state: days === 0 ? 'today' : 'future', ms }
      : { big: `${Math.floor(m / 60)}h ${m % 60}m`, small: '', short: `${Math.floor(m / 60)}h`, state: days === 0 ? 'today' : 'future', ms };
  }
  if (days === 0) return { big: 'Today', small: '', short: 'Today', state: 'today', ms };
  if (days === 1) return { big: 'Tomorrow', small: '', short: '1d', state: 'future', ms };
  const w = Math.floor(days / 7), r = days % 7;
  const small = w ? (r ? `${plural(w, 'week')} ${plural(r, 'day')}` : plural(w, 'week')) : '';
  return { big: `${days} days`, small, short: `${days}d`, state: 'future', ms };
}

/** How far from when it was added to the target, 0..1 (the dock ring and the dot row). */
export const progressOf = (created: number, target: Date, now: Date) => {
  const span = target.getTime() - created;
  return span <= 0 ? 1 : Math.min(1, Math.max(0, (now.getTime() - created) / span));
};
