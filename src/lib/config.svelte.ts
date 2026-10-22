// Shared reactive config. Rust owns the truth; every window mirrors it via `config:changed`.
import { events, ipc, type Config, type Profile } from './ipc';
import { applyTheme } from './theme';

export const store = $state<{ config: Config | null }>({ config: null });

export async function initConfig() {
  store.config = await ipc.getConfig();
  applyTheme(store.config.theme);
  await events.configChanged((c) => { store.config = c; applyTheme(c.theme); });
}

export const profile = (): Profile => store.config!.profiles[store.config!.activeProfile];

/** Clone, mutate, persist. Rust validates and broadcasts the result. */
export async function update(mutate: (c: Config) => void) {
  const next: Config = JSON.parse(JSON.stringify(store.config));
  mutate(next);
  store.config = next;
  await ipc.saveConfig(next);
}

export function updateWidgetSettings(instance: string, patch: Record<string, unknown>) {
  return update((c) => {
    const w = c.profiles[c.activeProfile].widgets.find((w) => w.instance === instance);
    if (w) w.settings = { ...(w.settings ?? {}), ...patch };
  });
}
