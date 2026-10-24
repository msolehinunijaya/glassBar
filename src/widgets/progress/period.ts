// How far through the current day / month / year we are, in local time.
// Uses real calendar boundaries, so month lengths, leap years and DST days are right.
export type Mode = 'day' | 'month' | 'year';

export function period(when: Date, mode: Mode): { frac: number; start: number; end: number; now: number } {
  const y = when.getFullYear(), m = when.getMonth(), d = when.getDate();
  const [start, end] =
    mode === 'day' ? [new Date(y, m, d), new Date(y, m, d + 1)]
    : mode === 'month' ? [new Date(y, m, 1), new Date(y, m + 1, 1)]
    : [new Date(y, 0, 1), new Date(y + 1, 0, 1)];
  const now = when.getTime();
  const frac = Math.min(1, Math.max(0, (now - start.getTime()) / (end.getTime() - start.getTime())));
  return { frac, start: start.getTime(), end: end.getTime(), now };
}

/** "3d 4h", "5h 12m", "12m" */
export function span(ms: number): string {
  const m = Math.max(0, Math.floor(ms / 60_000));
  const d = Math.floor(m / 1440), h = Math.floor((m % 1440) / 60), r = m % 60;
  if (d) return `${d}d ${h}h`;
  if (h) return `${h}h ${r}m`;
  return `${r}m`;
}

/** Dot grid from the reference design: one dot per week for the year (13×4),
 *  30 dots (10×3) for the month and the day. `filled` = elapsed dots. */
export function grid(mode: Mode, frac = 0) {
  const [cols, rows] = mode === 'year' ? [13, 4] : [10, 3];
  const total = cols * rows;
  return { cols, rows, total, filled: Math.round(Math.min(1, Math.max(0, frac)) * total) };
}

/** "Oct 2026" / "Oct 23" / "2026" */
export function label(d: Date, mode: Mode, locale?: string): string {
  const f = (o: Intl.DateTimeFormatOptions) => new Intl.DateTimeFormat(locale, o).format(d);
  return mode === 'year' ? String(d.getFullYear()) : mode === 'month' ? f({ month: 'short', year: 'numeric' }) : f({ month: 'short', day: 'numeric' });
}
