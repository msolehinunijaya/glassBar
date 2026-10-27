// App Folders: the pure part (which folder, which apps, in what order). Unit tested.
import type { Pin } from '../apps/index';
import { keyFor, nameOf } from '../apps/groups';

/** One folder on the dock. An empty name shows no label. */
export interface Folder { id: string; name: string; apps: Pin[] }

/** Popup keys: a folder opened by a click (focused) or by hovering (closes on mouse-out). */
export const FOLDER = '__folder:';
export const FOLDER_HOVER = '__folderh:';
export const folderKey = (hover: boolean, instance: string, id: string) => `${hover ? FOLDER_HOVER : FOLDER}${instance}|${id}`;
export function parseFolderKey(key: string | null | undefined): { instance: string; id: string; hover: boolean } | null {
  if (!key) return null;
  const hover = key.startsWith(FOLDER_HOVER);
  if (!hover && !key.startsWith(FOLDER)) return null;
  const [instance, id] = key.slice(hover ? FOLDER_HOVER.length : FOLDER.length).split('|');
  return instance && id ? { instance, id, hover } : null;
}

export const pinKey = (p: Pin) => keyFor(p.path, p.aumid);
export const pinName = (p: Pin) => p.name ?? nameOf(p.path);

/** Only programs and shortcuts can go in a folder. */
export const isApp = (path: string) => /\.(exe|lnk|url|appref-ms)$/i.test(path);

/** Add apps to the end, skipping ones already in the folder. */
export const addApps = (f: Folder, pins: Pin[]): Folder => {
  const have = new Set(f.apps.map(pinKey));
  const fresh = pins.filter((p) => { const k = pinKey(p); if (have.has(k)) return false; have.add(k); return true; });
  return { ...f, apps: [...f.apps, ...fresh] };
};
export const removeApp = (f: Folder, key: string): Folder => ({ ...f, apps: f.apps.filter((p) => pinKey(p) !== key) });
/** Move the app `key` to position `to` (clamped). */
export function moveApp(f: Folder, key: string, to: number): Folder {
  const from = f.apps.findIndex((p) => pinKey(p) === key);
  if (from < 0) return f;
  const apps = [...f.apps];
  const [p] = apps.splice(from, 1);
  apps.splice(Math.max(0, Math.min(apps.length, to)), 0, p);
  return { ...f, apps };
}
/** Replace folder `id` in the list. */
export const replace = (list: Folder[], f: Folder) => list.map((x) => (x.id === f.id ? f : x));
export const newFolder = (): Folder => ({ id: `f${Date.now().toString(36)}${Math.random().toString(36).slice(2, 5)}`, name: '', apps: [] });
