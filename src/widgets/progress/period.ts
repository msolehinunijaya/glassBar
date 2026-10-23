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
