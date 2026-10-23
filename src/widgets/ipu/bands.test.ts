import { expect, it } from 'vitest';
import { BANDS } from './bands';

it('uses the official IPU status colours and names', () => {
  expect(BANDS.good).toEqual({ name: 'Good', color: '#1e88e5' });
  expect(BANDS.moderate.name).toBe('Moderate');
  expect(BANDS.moderate.color).toBe('#43a047');
  expect(BANDS.unhealthy.color).toBe('#fdd835');
  expect(BANDS.very_unhealthy.color).toBe('#fb8c00');
  expect(BANDS.hazardous.color).toBe('#e53935');
});

it('bandOf matches the Rust thresholds', async () => {
  const { bandOf } = await import('./bands');
  expect([50, 51, 100, 101, 201, 301].map(bandOf)).toEqual(['good', 'moderate', 'moderate', 'unhealthy', 'very_unhealthy', 'hazardous']);
});
