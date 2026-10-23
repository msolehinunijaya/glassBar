import { describe, expect, it } from 'vitest';
import { group, nextWindow, nameOf } from './groups';

const win = (id: number, exe: string, active = false) => ({ id, title: `w${id}`, exe, active, minimized: false });

describe('group', () => {
  it('puts pins first and merges running windows into them case-insensitively', () => {
    const g = group([{ path: 'C:\\Apps\\Code.exe' }], [win(1, 'C:\\Win\\notepad.exe'), win(2, 'c:\\apps\\code.exe')]);
    expect(g.map((x) => x.name)).toEqual(['Code', 'notepad']);
    expect(g[0].windows.map((w) => w.id)).toEqual([2]);
  });
  it('groups several windows of one program and skips unknown exes', () => {
    const g = group([], [win(1, 'a.exe'), win(2, ''), win(3, 'a.exe')]);
    expect(g).toHaveLength(1);
    expect(g[0].windows).toHaveLength(2);
  });
});

describe('nextWindow', () => {
  it('cycles when the group is active, else picks the first', () => {
    const [g] = group([], [win(1, 'a.exe'), win(2, 'a.exe', true), win(3, 'a.exe')]);
    expect(nextWindow(g)?.id).toBe(3);
    const [h] = group([], [win(1, 'a.exe'), win(2, 'a.exe')]);
    expect(nextWindow(h)?.id).toBe(1);
    const [solo] = group([], [win(7, 'a.exe', true)]);
    expect(nextWindow(solo)?.id).toBe(7); // same window: Rust minimizes it
  });
});

it('nameOf strips folders and extensions', () => {
  expect(nameOf('C:\\x\\Spotify.exe')).toBe('Spotify');
  expect(nameOf('C:/y/Chrome.lnk')).toBe('Chrome');
});

describe('stableOrder', () => {
  it('keeps the order apps were first seen, whatever Windows z-order says', async () => {
    const { stableOrder } = await import('./groups');
    let order: string[] = [];
    order = stableOrder(order, ['a', 'b', 'c']);
    expect(order).toEqual(['a', 'b', 'c']);
    order = stableOrder(order, ['c', 'a', 'b']); // focusing c moves it to the top of z-order
    expect(order).toEqual(['a', 'b', 'c']);
    order = stableOrder(order, ['d', 'a', 'c']); // b closed, d opened
    expect(order).toEqual(['a', 'c', 'd']);
  });
});
