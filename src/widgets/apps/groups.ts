// Pure grouping logic for the launcher (unit tested).
import type { AppWindow } from '../../lib/ipc';
import type { Pin } from './index';

export interface Group { key: string; path: string; name: string; pin?: Pin; windows: AppWindow[] }

export const keyOf = (path: string) => path.toLowerCase();
export const nameOf = (path: string) => path.split(/[\\/]/).pop()?.replace(/\.(exe|lnk)$/i, '') ?? path;

/** Pinned apps first (in pin order), then other running programs in first-seen order. */
export function group(pins: Pin[], windows: AppWindow[]): Group[] {
  const out = new Map<string, Group>();
  for (const pin of pins) out.set(keyOf(pin.path), { key: keyOf(pin.path), path: pin.path, name: nameOf(pin.path), pin, windows: [] });
  for (const w of windows) {
    if (!w.exe) continue;
    const k = keyOf(w.exe);
    if (!out.has(k)) out.set(k, { key: k, path: w.exe, name: nameOf(w.exe), windows: [] });
    out.get(k)!.windows.push(w);
  }
  return [...out.values()];
}

/** Which window a click on a group should act on: cycle through windows if one is active. */
export function nextWindow(g: Group): AppWindow | undefined {
  const i = g.windows.findIndex((w) => w.active);
  return i >= 0 && g.windows.length > 1 ? g.windows[(i + 1) % g.windows.length] : g.windows[Math.max(i, 0)];
}
