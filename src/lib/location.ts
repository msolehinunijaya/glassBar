// Pure helpers for the shared location (badge, labels, validation). See location.svelte.ts for the live value.
import type { LocationStatus, Place } from './ipc';

export type Badge = { kind: 'precise' | 'approx' | 'manual' | 'none'; label: string; short: string };

/** How the current location was found. */
export function badge(s: Pick<LocationStatus, 'place'> | null): Badge {
  switch (s?.place?.source) {
    case 'windows': return { kind: 'precise', label: 'Precise (Windows Location)', short: 'Precise' };
    case 'ip': return { kind: 'approx', label: 'Approximate (IP)', short: 'Approx (IP)' };
    case 'manual': return { kind: 'manual', label: 'Manual', short: 'Manual' };
    default: return { kind: 'none', label: 'Unavailable', short: 'Unavailable' };
  }
}

/** Accuracy radius as reported; never guessed. */
export const accuracyText = (m: number | undefined) =>
  m == null ? 'not reported' : m >= 1000 ? `${(m / 1000).toFixed(1)} km` : `${Math.round(m)} m`;

export const validLatLon = (lat: number, lon: number) =>
  Number.isFinite(lat) && Number.isFinite(lon) && lat >= -90 && lat <= 90 && lon >= -180 && lon <= 180;

export const coords = (p: Place) => `${p.lat.toFixed(4)}, ${p.lon.toFixed(4)}`;

/** First part of the place name ("Kuala Lumpur"), or the coordinates when it has none. */
export const placeShort = (p: Place) => p.name.split(',')[0].trim() || coords(p);

/** "Kuala Lumpur · Precise" for widget panels. */
export const tagText = (s: LocationStatus | null) => (s?.place ? `${placeShort(s.place)} · ${badge(s).short}` : 'Location unavailable');
