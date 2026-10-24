import { describe, expect, it } from 'vitest';
import { describe as wx } from './codes';

describe('WMO weather codes', () => {
  it('maps the main groups', () => {
    expect(wx(0, true).label).toBe('Clear');
    expect(wx(61, true).label).toBe('Rain');
    expect(wx(95, true).label).toBe('Thunderstorm');
    expect(wx(3, true).label).toBe('Cloudy');
  });
  it('uses a moon at night for clear skies', () => {
    expect(wx(0, false).icon).toBe('moon');
    expect(wx(0, true).icon).toBe('sun');
  });
  it('falls back for unknown codes', () => {
    expect(wx(1234, true).label).toBe('Unknown');
  });
});
