// Pure grouping logic for the launcher (unit tested).
import type { AppWindow } from '../../lib/ipc';
import type { Pin } from './index';

export interface Group { key: string; path: string; name: string; pin?: Pin; windows: AppWindow[]; aumid?: string }

/** Program identity. Store apps' folders carry a version (…_1.2.3.0_x64__publisher) that
 *  changes on every update: drop it so a pin keeps matching the running app. */
export const keyOf = (path: string) =>
  path.toLowerCase().replace(/(\\windowsapps\\[^\\_]+)_[^\\]*?__/, '$1__');
export const nameOf = (path: string) => path.split(/[\\/]/).pop()?.replace(/\.(exe|lnk)$/i, '') ?? path;

/** Pinned apps first (in pin order), then other running programs in first-seen order. */
export function group(pins: Pin[], windows: AppWindow[]): Group[] {
  const out = new Map<string, Group>();
  for (const pin of pins) out.set(keyOf(pin.path), { key: keyOf(pin.path), path: pin.path, name: nameOf(pin.path), pin, windows: [], aumid: pin.aumid });
  for (const w of windows) {
    if (!w.exe) continue;
    const k = keyOf(w.exe);
    if (!out.has(k)) out.set(k, { key: k, path: w.exe, name: nameOf(w.exe), windows: [] });
    const g = out.get(k)!;
    g.windows.push(w);
    g.aumid ??= w.aumid ?? undefined;
  }
  return [...out.values()];
}

/** Which window a click on a group should act on: cycle through windows if one is active. */
export function nextWindow(g: Group): AppWindow | undefined {
  const i = g.windows.findIndex((w) => w.active);
  return i >= 0 && g.windows.length > 1 ? g.windows[(i + 1) % g.windows.length] : g.windows[Math.max(i, 0)];
}

/** Keep keys in the order they were first seen: new ones go to the end, gone ones drop out.
 *  (Windows lists windows in z-order, which changes on every focus.) */
export function stableOrder(prev: string[], current: string[]): string[] {
  const now = new Set(current);
  const kept = prev.filter((k) => now.has(k));
  const known = new Set(kept);
  return [...kept, ...current.filter((k) => !known.has(k))];
}

/** Move `key` so it ends up at index `to` (unknown keys: no change). */
export function moveKey(order: string[], key: string, to: number): string[] {
  const i = order.indexOf(key);
  if (i < 0) return order;
  const rest = order.filter((k) => k !== key);
  rest.splice(Math.max(0, Math.min(to, rest.length)), 0, key);
  return rest;
}

/** Pins sorted by the user's saved icon order; pins not in it keep their place at the end. */
export function pinsInOrder(pins: Pin[], order: string[]): Pin[] {
  const at = (p: Pin) => { const i = order.indexOf(keyOf(p.path)); return i < 0 ? Infinity : i; };
  return [...pins].sort((a, b) => at(a) - at(b));
}
