// Time and date formatting for the clock. Month and weekday names come from Intl in the
// chosen (or system) language; nothing is hard-coded. Pure, so it's unit tested.

export type AmPm = 'PM' | 'pm' | 'P.M.' | 'p.m.';
export interface TimeOpts { h12: boolean; seconds: boolean; leadingZero: boolean; ampm: AmPm }

export type DateStyle = 'short' | 'dmy' | 'mdy' | 'iso' | 'medium' | 'weekday' | 'long' | 'custom';
export interface DateOpts {
  style: DateStyle;
  /** Separator for the numeric styles (ISO always uses "-"). */
  sep: '/' | '-' | '.';
  /** Add (or force) a weekday in front: Fri / Friday. */
  weekday: 'none' | 'short' | 'full';
  /** Tokens: D DD M MM MMM MMMM YY YYYY ddd dddd; text in [brackets] is literal. */
  pattern: string;
  locale?: string;
}

const pad = (n: number) => String(n).padStart(2, '0');

/** Hour/minute/second/date parts in `timeZone` (default: local). */
function parts(d: Date, timeZone?: string) {
  if (!timeZone) return { y: d.getFullYear(), mo: d.getMonth() + 1, day: d.getDate(), h: d.getHours(), mi: d.getMinutes(), s: d.getSeconds(), wd: d.getDay() };
  const p = Object.fromEntries(new Intl.DateTimeFormat('en-US', {
    timeZone, hourCycle: 'h23', year: 'numeric', month: 'numeric', day: 'numeric', hour: 'numeric', minute: 'numeric', second: 'numeric', weekday: 'short',
  }).formatToParts(d).map((x) => [x.type, x.value]));
  const wd = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'].indexOf(p.weekday);
  return { y: +p.year, mo: +p.month, day: +p.day, h: +p.hour % 24, mi: +p.minute, s: +p.second, wd };
}

/** "14:05", "2:05:09" + ampm "PM" (kept apart so it can be shown smaller or hidden). */
export function formatTime(d: Date, o: TimeOpts, timeZone?: string): { main: string; ampm: string } {
  let p;
  try { p = parts(d, timeZone); } catch { return { main: '--:--', ampm: '' }; } // unknown time zone
  let h = p.h;
  let ampm = '';
  if (o.h12) {
    const pm = h >= 12;
    h = h % 12 || 12;
    const base = pm ? 'PM' : 'AM';
    ampm = o.ampm === 'pm' ? base.toLowerCase() : o.ampm === 'P.M.' ? `${base[0]}.M.` : o.ampm === 'p.m.' ? `${base[0].toLowerCase()}.m.` : base;
  }
  const hh = o.leadingZero ? pad(h) : String(h);
  return { main: `${hh}:${pad(p.mi)}${o.seconds ? `:${pad(p.s)}` : ''}`, ampm };
}

const TOKENS = /\[[^\]]*\]|dddd|ddd|DD|D|MMMM|MMM|MM|M|YYYY|YY/g;

/** A pattern must use at least one token and have no stray letters (likely a typo). */
export function isValidPattern(pattern: string): boolean {
  if (!pattern.trim()) return false;
  const rest = pattern.replace(TOKENS, '');
  return rest.length < pattern.length && !/[A-Za-z]/.test(rest);
}

function names(locale: string | undefined, timeZone: string | undefined, d: Date) {
  const f = (o: Intl.DateTimeFormatOptions) => new Intl.DateTimeFormat(locale, { ...o, timeZone }).format(d);
  return { ddd: f({ weekday: 'short' }), dddd: f({ weekday: 'long' }), MMM: f({ month: 'short' }), MMMM: f({ month: 'long' }) };
}

function applyPattern(d: Date, pattern: string, locale?: string, timeZone?: string): string {
  const p = parts(d, timeZone);
  const n = names(locale, timeZone, d);
  const v: Record<string, string> = {
    D: String(p.day), DD: pad(p.day), M: String(p.mo), MM: pad(p.mo), YY: pad(p.y % 100), YYYY: String(p.y),
    ddd: n.ddd, dddd: n.dddd, MMM: n.MMM, MMMM: n.MMMM,
  };
  return pattern.replace(TOKENS, (t) => (t.startsWith('[') ? t.slice(1, -1) : v[t]));
}

const PRESETS: Record<Exclude<DateStyle, 'custom'>, (sep: string) => string> = {
  short: (s) => `DD${s}MM`,
  dmy: (s) => `DD${s}MM${s}YYYY`,
  mdy: (s) => `MM${s}DD${s}YYYY`,
  iso: () => 'YYYY-MM-DD',
  medium: () => 'D MMM YYYY',
  weekday: () => 'ddd, D MMM',
  long: () => 'dddd, D MMMM YYYY',
};

export function formatDate(d: Date, o: DateOpts, timeZone?: string): string {
  let pattern: string;
  if (o.style === 'custom') pattern = isValidPattern(o.pattern) ? o.pattern : PRESETS.weekday(o.sep);
  else pattern = PRESETS[o.style](o.sep);
  if (o.weekday !== 'none') {
    const w = o.weekday === 'full' ? 'dddd' : 'ddd';
    pattern = pattern.replace(/^dddd?, /, '');
    pattern = `${w}, ${pattern}`;
  }
  return applyPattern(d, pattern, o.locale, timeZone);
}
