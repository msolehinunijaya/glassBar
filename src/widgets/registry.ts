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

/** Append a new instance of widget `id` to the active profile. */
export const addWidget = (id: string) =>
  update((c) => {
    c.profiles[c.activeProfile].widgets.push({
      id, instance: `${id}-${Date.now().toString(36)}`, size: widgets[id].defaultSize, settings: {},
    });
  });
