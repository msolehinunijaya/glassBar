import type { Band } from '../../lib/ipc';

// IPU status colours: blue, green, yellow, orange, red.
// `short` fits a left/right dock tile.
export const BANDS: Record<Band, { name: string; short: string; color: string }> = {
  good: { name: 'Good', short: 'Good', color: '#1e88e5' },
  moderate: { name: 'Moderate', short: 'Moderate', color: '#43a047' },
  unhealthy: { name: 'Unhealthy', short: 'Unhealthy', color: '#fdd835' },
  very_unhealthy: { name: 'Very Unhealthy', short: 'V. unhealthy', color: '#fb8c00' },
  hazardous: { name: 'Hazardous', short: 'Hazardous', color: '#e53935' },
};

/** Same thresholds as Rust's `band()`. */
export const bandOf = (v: number): Band =>
  v <= 50 ? 'good' : v <= 100 ? 'moderate' : v <= 200 ? 'unhealthy' : v <= 300 ? 'very_unhealthy' : 'hazardous';
