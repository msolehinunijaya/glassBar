import { describe, expect, it } from 'vitest';
import type { WidgetInst, WidgetSize } from '../lib/ipc';
import { categoryOf } from './categories';
import { applyLayout, counts, isOn, matches, stateOf, visibleOnDock, type Query } from './layout';

const all: WidgetSize[] = ['small', 'medium', 'large'];
const known = {
  start: { defaultSize: 'small' as const, sizes: ['small' as const] },
  apps: { defaultSize: 'large' as const, sizes: all },
  clock: { defaultSize: 'medium' as const, sizes: all },
  music: { defaultSize: 'medium' as const, sizes: all },
};
const W = (id: string, extra: Partial<WidgetInst> = {}): WidgetInst => ({ id, instance: `${id}-1`, size: 'medium', settings: { mine: id }, ...extra });

describe('applyLayout', () => {
  it('turns the listed widgets on in order and keeps the rest, off, with their settings', () => {
    const out = applyLayout([W('music'), W('apps', { size: 'small' }), W('clock')], [{ id: 'clock' }, { id: 'apps' }], known);
    expect(out.map((w) => [w.id, isOn(w)])).toEqual([['clock', true], ['apps', true], ['music', false]]);
    expect(out[1].size).toBe('small');
    expect(out[2].settings).toEqual({ mine: 'music' });
  });
  it('adds missing widgets with defaults, applies preset sizes, never duplicates, skips unknown ids', () => {
    const out = applyLayout([W('apps')], [{ id: 'start' }, { id: 'apps', size: 'large' }, { id: 'apps' }, { id: 'nope' }], known, 36);
    expect(out.map((w) => w.id)).toEqual(['start', 'apps']);
    expect(out[0]).toMatchObject({ instance: 'start-10', size: 'small', enabled: true, settings: {} });
    expect(out[1].size).toBe('large');
  });
  it('ignores a size the widget does not support', () => {
    expect(applyLayout([], [{ id: 'start', size: 'large' }], known)[0].size).toBe('small');
  });
});

describe('categories and filters', () => {
  const ms = [
    { id: 'clock', name: 'Clock & Date', description: 'time', category: 'productivity' },
    { id: 'music', name: 'Music', description: 'now playing', category: 'media' },
    { id: 'new', name: 'Brand new', description: 'x', category: 'made-up' },
  ];
  const placed = [W('clock'), W('music', { enabled: false })];
  const f = (q: Partial<Query>) => ms.filter((m) => matches(m, placed, { text: '', category: 'all', filter: 'all', ...q })).map((m) => m.id);

  it('puts unknown categories in Other', () => expect(categoryOf(ms[2])).toBe('other'));
  it('knows enabled, disabled and available', () => {
    expect(['clock', 'music', 'new'].map((id) => stateOf(id, placed))).toEqual(['enabled', 'disabled', 'available']);
  });
  it('filters by text, category and state', () => {
    expect(f({ text: 'PLAYING' })).toEqual(['music']);
    expect(f({ category: 'media' })).toEqual(['music']);
    expect(f({ filter: 'available' })).toEqual(['new']);
    expect(f({ filter: 'disabled' })).toEqual(['music']);
  });
  it('counts per category', () => {
    expect(counts(ms, placed)).toEqual({ productivity: { total: 1, on: 1 }, media: { total: 1, on: 0 }, other: { total: 1, on: 0 } });
  });
});

describe('visibleOnDock', () => {
  it('shows widgets that are on, and hides Start while the launcher shows it inline', () => {
    const k = { start: 1, apps: 1, clock: 1 };
    const ids = (l: WidgetInst[]) => visibleOnDock(l, k).map((w) => w.id);
    expect(ids([W('start'), W('apps', { settings: {} }), W('clock', { enabled: false }), W('gone')])).toEqual(['apps']);
    expect(ids([W('start'), W('apps', { settings: { startPlacement: 'separate' } })])).toEqual(['start', 'apps']);
    expect(ids([W('start'), W('apps', { enabled: false })])).toEqual(['start']);
  });
});
