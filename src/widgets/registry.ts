// Auto-discovers every src/widgets/<id>/index.ts. No central list to edit.
// Folders starting with "_" (the SDK template) only show up in dev builds.
import type { WidgetManifest } from './types';
import { update } from '../lib/config.svelte';

const found = import.meta.glob<{ default: WidgetManifest }>('./*/index.ts', { eager: true });

export const widgets: Record<string, WidgetManifest> = Object.fromEntries(
  Object.entries(found)
    .filter(([path]) => import.meta.env.DEV || !path.startsWith('./_'))
    .map(([, mod]) => [mod.default.id, mod.default]),
);

export { settingsOf } from './types';

/** Widgets not yet on the active profile's bar (each widget can be placed once). */
export const available = (placed: { id: string }[]) =>
  Object.values(widgets).filter((m) => !placed.some((w) => w.id === m.id));

/** Append widget `id` to the active profile, unless it is already there. */
export const addWidget = (id: string) =>
  update((c) => {
    if (c.profiles[c.activeProfile].widgets.some((w) => w.id === id)) return;
    c.profiles[c.activeProfile].widgets.push({
      id, instance: `${id}-${Date.now().toString(36)}`, size: widgets[id].defaultSize, settings: {},
    });
  });
