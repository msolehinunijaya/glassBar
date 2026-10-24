// Auto-discovers every src/widgets/<id>/index.ts. No central list to edit.
// Folders whose index has no default export are parts of another widget (e.g. todo).
// Folders starting with "_" (the SDK template) only show up in dev builds.
import type { WidgetManifest } from './types';
import { update } from '../lib/config.svelte';

const found = import.meta.glob<{ default: WidgetManifest }>('./*/index.ts', { eager: true });

export const widgets: Record<string, WidgetManifest> = Object.fromEntries(
  Object.entries(found)
    .filter(([path, mod]) => mod.default && (import.meta.env.DEV || !path.startsWith('./_')))
    .map(([, mod]) => [mod.default.id, mod.default]),
);

export { settingsOf } from './types';

/** Widgets not on the active profile's dock: never added, or switched off (one of each kind). */
export const available = (placed: { id: string; enabled?: boolean }[]) =>
  Object.values(widgets).filter((m) => !placed.some((w) => w.id === m.id && w.enabled !== false));

/** Put widget `id` on the active profile's dock: switch it back on if it is there but off,
 *  else append it with defaults. */
export const addWidget = (id: string) =>
  update((c) => {
    const had = c.profiles[c.activeProfile].widgets.find((w) => w.id === id);
    if (had) { had.enabled = true; return; }
    c.profiles[c.activeProfile].widgets.push({
      id, instance: `${id}-${Date.now().toString(36)}`, size: widgets[id].defaultSize, settings: {},
    });
  });
