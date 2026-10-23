import { expect, it, vi } from 'vitest';
vi.mock('../../lib/ipc', () => ({ ipc: {}, events: {} }));
vi.mock('../../lib/live.svelte', () => ({ live: () => ({ v: [] }) }));
import { split } from './icons.svelte';

const icon = (id: string, promoted: boolean) => ({ id, tip: id, icon: null, exe: id, promoted, rev: 0 });

it('puts promoted icons on the dock and the rest in overflow', () => {
  const all = [icon('a', true), icon('b', false), icon('c', true)];
  const s = split(all, { showPromoted: true, compact: false });
  expect(s.inline.map((i) => i.id)).toEqual(['a', 'c']);
  expect(s.overflow.map((i) => i.id)).toEqual(['b']);
  expect(split(all, { showPromoted: false, compact: false }).inline).toEqual([]);
});

it('compact mode keeps at most 4 on the dock', () => {
  const all = Array.from({ length: 6 }, (_, i) => icon(String(i), true));
  const s = split(all, { showPromoted: true, compact: true });
  expect(s.inline).toHaveLength(4);
  expect(s.overflow.map((i) => i.id)).toEqual(['4', '5']);
});
