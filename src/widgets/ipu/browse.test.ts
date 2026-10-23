import { describe, expect, it } from 'vitest';
import type { Station } from '../../lib/ipc';
import { statesOf } from './browse';

const st = (id: string, name: string, state: string, api: number | null = 50): Station =>
  ({ id, name, state, lat: 0, lon: 0, api, class: '', pollutant: 'PM2.5', at: 0 });

describe('statesOf', () => {
  it('groups stations by state, sorted, skipping stations without a reading', () => {
    const g = statesOf([st('a', 'Klang', 'Selangor'), st('b', 'Cheras', 'WP Kuala Lumpur'), st('c', 'Banting', 'Selangor'), st('d', 'Off', 'Perlis', null)]);
    expect(g.map(([s]) => s)).toEqual(['Selangor', 'WP Kuala Lumpur']);
    expect(g[0][1].map((x) => x.name)).toEqual(['Banting', 'Klang']);
  });
});
