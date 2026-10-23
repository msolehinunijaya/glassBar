// Sunrise and sunset, worked out offline from latitude/longitude and the date with the
// standard sunrise equation (accurate to a few minutes; no network call).
const RAD = Math.PI / 180;
const J2000 = 2451545;
const julian = (ms: number) => ms / 86400000 + 2440587.5;
const fromJulian = (j: number) => (j - 2440587.5) * 86400000;

export interface SunTimes { rise: number | null; set: number | null; polar?: 'day' | 'night' }

/** Sunrise/sunset (UTC ms) for the local calendar day y-m-d at that longitude. */
export function sunTimes(lat: number, lon: number, y: number, m: number, d: number): SunTimes {
  // Days since J2000 for this date (Julian day at 00:00 UTC, rounded up as the equation expects).
  const n = Math.ceil(julian(Date.UTC(y, m - 1, d)) - J2000 + 0.0008);
  const jStar = n - lon / 360;
  const M = (357.5291 + 0.98560028 * jStar) % 360;
  const C = 1.9148 * Math.sin(M * RAD) + 0.02 * Math.sin(2 * M * RAD) + 0.0003 * Math.sin(3 * M * RAD);
  const lambda = (M + C + 180 + 102.9372) % 360;
  const transit = J2000 + jStar + 0.0053 * Math.sin(M * RAD) - 0.0069 * Math.sin(2 * lambda * RAD);
  const sinDec = Math.sin(lambda * RAD) * Math.sin(23.4397 * RAD);
  const cosDec = Math.cos(Math.asin(sinDec));
  const cosW = (Math.sin(-0.833 * RAD) - Math.sin(lat * RAD) * sinDec) / (Math.cos(lat * RAD) * cosDec);
  if (cosW < -1) return { rise: null, set: null, polar: 'day' };
  if (cosW > 1) return { rise: null, set: null, polar: 'night' };
  const w = Math.acos(cosW) / RAD / 360;
  return { rise: fromJulian(transit - w), set: fromJulian(transit + w) };
}

/** The calendar date it is in `tz` at `ms`. */
function localDate(ms: number, tz: string) {
  const p = Object.fromEntries(new Intl.DateTimeFormat('en-US', { timeZone: tz, year: 'numeric', month: 'numeric', day: 'numeric' })
    .formatToParts(new Date(ms)).map((x) => [x.type, +x.value]));
  return { y: p.year, m: p.month, d: p.day };
}
const localHour = (ms: number, tz: string) =>
  +new Intl.DateTimeFormat('en-US', { timeZone: tz, hourCycle: 'h23', hour: 'numeric' }).format(new Date(ms)) % 24;

export type DayState = 'day' | 'night' | 'sunrise' | 'sunset';
export interface DayInfo { state: DayState; nextRise: number | null; nextSet: number | null }

const TWILIGHT = 30 * 60000;

/** Day/night for a world clock now. Without coordinates: 6 AM–6 PM local is day. */
export function dayState(now: number, tz: string, coords?: { lat: number; lon: number }): DayInfo {
  if (!coords) {
    const h = localHour(now, tz);
    return { state: h >= 6 && h < 18 ? 'day' : 'night', nextRise: null, nextSet: null };
  }
  const { y, m, d } = localDate(now, tz);
  const today = sunTimes(coords.lat, coords.lon, y, m, d);
  if (today.polar) return { state: today.polar, nextRise: null, nextSet: null };
  const { rise, set } = today as { rise: number; set: number };
  let state: DayState = now >= rise && now < set ? 'day' : 'night';
  if (Math.abs(now - rise) <= TWILIGHT) state = 'sunrise';
  else if (Math.abs(now - set) <= TWILIGHT) state = 'sunset';
  // After sunset the next sunrise is tomorrow's.
  let nextRise: number | null = rise;
  if (now > rise) {
    const t = new Date(Date.UTC(y, m - 1, d + 1));
    nextRise = sunTimes(coords.lat, coords.lon, t.getUTCFullYear(), t.getUTCMonth() + 1, t.getUTCDate()).rise;
  }
  return { state, nextRise, nextSet: now < set ? set : null };
}
