import { describe, expect, it } from 'vitest';
import { accuracyText, badge, placeShort, tagText, validLatLon } from './location';
import type { LocationStatus, Place } from './ipc';

const P = (source: Place['source'], name = 'Kuala Lumpur, Wilayah Persekutuan, Malaysia'): Place => ({ lat: 3.1478, lon: 101.6953, name, source });
const S = (place: Place | null): LocationStatus => ({ mode: 'auto', place, reason: place ? null : 'offline', lastKnown: null, at: 0 });

describe('location badge', () => {
  it('maps each source to its label', () => {
    expect(badge(S(P('windows'))).label).toBe('Precise (Windows Location)');
    expect(badge(S(P('ip'))).label).toBe('Approximate (IP)');
    expect(badge(S(P('manual'))).label).toBe('Manual');
    expect(badge(S(null)).label).toBe('Unavailable');
    expect(badge(null).kind).toBe('none');
  });
  it('builds the widget tag', () => {
    expect(tagText(S(P('windows')))).toBe('Kuala Lumpur · Precise');
    expect(tagText(S(P('ip', 'Selangor')))).toBe('Selangor · Approx (IP)');
    expect(tagText(S(null))).toBe('Location unavailable');
    expect(placeShort(P('windows', ''))).toBe('3.1478, 101.6953');
  });
});

describe('location values', () => {
  it('validates latitude and longitude ranges', () => {
    expect(validLatLon(90, 180)).toBe(true);
    expect(validLatLon(-90, -180)).toBe(true);
    expect(validLatLon(90.01, 0)).toBe(false);
    expect(validLatLon(0, -180.5)).toBe(false);
    expect(validLatLon(NaN, 0)).toBe(false);
  });
  it('shows accuracy only as reported', () => {
    expect(accuracyText(undefined)).toBe('not reported');
    expect(accuracyText(35)).toBe('35 m');
    expect(accuracyText(1500)).toBe('1.5 km');
  });
});
