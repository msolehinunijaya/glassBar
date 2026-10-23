// Turns the clock's settings into formatter options (shared by the pill and the panel).
import { ipc } from '../../lib/ipc';
import { formatDate, formatTime, type DateOpts, type TimeOpts } from './format';
import type { ClockSettings } from './index';

// Windows' regional 12/24-hour choice, read once.
const system = $state<{ h12: boolean | null }>({ h12: null });
let asked = false;
function systemH12() {
  if (!asked) { asked = true; ipc.clockSystem12h().then((v) => (system.h12 = v)).catch(() => {}); }
  return system.h12;
}

export function is12h(s: ClockSettings): boolean {
  const f = s.timeFormat ?? (s.hour12 ? '12' : 'system');
  return f === '12' || (f === 'system' && systemH12() === true);
}

export function timeOpts(s: ClockSettings, seconds = s.showSeconds): TimeOpts {
  const h12 = is12h(s);
  return { h12, seconds, ampm: s.ampm, leadingZero: s.leadingZero === 'on' || (s.leadingZero === 'auto' && !h12) };
}

export const dateOpts = (s: ClockSettings): DateOpts => ({ style: s.dateStyle, sep: s.dateSep, weekday: s.weekday, pattern: s.datePattern });

export const time = (d: Date, s: ClockSettings, seconds?: boolean, tz?: string) => formatTime(d, timeOpts(s, seconds), tz);
export const date = (d: Date, s: ClockSettings, tz?: string) => formatDate(d, dateOpts(s), tz);
/** One string, e.g. for tooltips: "2:05 PM" (AM/PM kept unless hidden). */
export const timeText = (d: Date, s: ClockSettings, seconds = false, tz?: string) => {
  const t = time(d, s, seconds, tz);
  return t.ampm && s.ampmSize !== 'hidden' ? `${t.main} ${t.ampm}` : t.main;
};
