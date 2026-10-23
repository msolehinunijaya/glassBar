// Small helpers for side-dock tiles (left/right dock).

/** Meter colour for a 0–100 load: calm, busy (≥ 60), high (≥ 85). */
export const loadTone = (pct: number) => (pct >= 85 ? '#ff453a' : pct >= 60 ? '#ff9f0a' : 'var(--accent)');

/** Bytes per second in at most 4 characters + unit letter: "12", "978K", "1.1M", "24M", "1.2G". */
export function rateShort(bps: number): string {
  const units = ['', 'K', 'M', 'G'];
  let v = Math.max(0, bps), i = 0;
  while (v >= 999.5 && i < units.length - 1) { v /= 1024; i++; }
  const n = v < 9.95 && i > 0 ? v.toFixed(1) : String(Math.round(v));
  return n + units[i];
}
