import type { Icon } from './codes';

export type SunPhase = 'before' | 'day' | 'after' | 'none';

/** Where the sun is on today's arc: 0 = sunrise, 1 = sunset. 'none' = no data (polar). */
export function sunArc(now: number, rise: number | null | undefined, set: number | null | undefined): { frac: number; phase: SunPhase } {
  if (rise == null || set == null || set <= rise) return { frac: 0, phase: 'none' };
  if (now < rise) return { frac: 0, phase: 'before' };
  if (now > set) return { frac: 1, phase: 'after' };
  return { frac: (now - rise) / (set - rise), phase: 'day' };
}

export function daylight(rise: number | null | undefined, set: number | null | undefined): string | null {
  if (rise == null || set == null || set <= rise) return null;
  const m = Math.round((set - rise) / 60_000);
  return `${Math.floor(m / 60)}h ${m % 60}m`;
}

const POINTS = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW'];
export const compass = (deg: number) => POINTS[Math.round((((deg % 360) + 360) % 360) / 45) % 8];

/** Background gradient for the pill and dashboard, by condition and time of day. */
export function themeOf(icon: Icon, isDay: boolean): string {
  if (!isDay) return icon === 'storm' ? 'linear-gradient(135deg,#2b2340,#4a3f6b)' : icon === 'rain' || icon === 'drizzle'
    ? 'linear-gradient(135deg,#1c2a3f,#33475f)' : 'linear-gradient(135deg,#141e3c,#2d3a6b)';
  switch (icon) {
    case 'sun': return 'linear-gradient(135deg,#ffb340,#ff7a45)';
    case 'cloud-sun': return 'linear-gradient(135deg,#5cb6ff,#ffc66b)';
    case 'cloud': case 'fog': return 'linear-gradient(135deg,#8e9aaf,#b9c2d0)';
    case 'drizzle': case 'rain': return 'linear-gradient(135deg,#3f6fb5,#6b93c9)';
    case 'snow': return 'linear-gradient(135deg,#9ec9ef,#dbe9f7)';
    case 'storm': return 'linear-gradient(135deg,#4b3f73,#7a6aa8)';
    default: return 'linear-gradient(135deg,#5cb6ff,#8ec5ff)';
  }
}
