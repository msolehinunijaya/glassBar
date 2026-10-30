// Shared reactive config. Rust owns the truth; every window mirrors it via `config:changed`.
import { emit, listen } from '@tauri-apps/api/event';
import { events, ipc, type Config, type Profile, type Theme } from './ipc';
import { applyTheme } from './theme';

export const store = $state<{ config: Config | null }>({ config: null });

/** This window's label: a window ignores its own preview broadcasts. */
const SELF = typeof window === 'undefined' ? '' : (document.documentElement.dataset.window ?? '') + ':' + Math.random().toString(36).slice(2);

export async function initConfig() {
  store.config = await ipc.getConfig();
  applyTheme(store.config.theme);
  // While this window has a change waiting to be saved, an older saved copy from Rust must not
  // flip the setting back for a moment.
  await events.configChanged((c) => { if (pending) return; store.config = c; applyTheme(c.theme); });
  // Another window changed a setting (a slider being dragged): show it here at once.
  await listen<{ from: string; config: Config }>('config:preview', (e) => {
    if (e.payload.from === SELF) return;
    store.config = e.payload.config;
    applyTheme(e.payload.config.theme);
  });
  await listen<Theme>('appearance:preview', (e) => applyTheme(e.payload));
}

/** Restyle every window at once without saving (a slider being dragged). The save on
 *  release then makes it stick; a closed Settings window leaves the saved look. */
export const previewTheme = (t: Theme) => emit('appearance:preview', t);

export const profile = (): Profile => store.config!.profiles[store.config!.activeProfile];

// Saving is the heavy part (Rust writes the file atomically and re-applies the dock), so a
// burst of changes -- a slider being dragged, the mouse wheel on one -- shows at once in
// every window (`config:preview`) and is saved once it settles.
const SAVE_AFTER_MS = 150;
let pending: Config | null = null;
let saveTimer: ReturnType<typeof setTimeout> | undefined;
let waiting: (() => void)[] = [];

async function flush() {
  const c = pending;
  const done = waiting;
  pending = null;
  waiting = [];
  if (!c) return;
  try {
    await ipc.saveConfig(c);
  } catch (e) {
    store.config = await ipc.getConfig(); // Rust rejected it: show what is really saved
    console.error('save_config failed', e);
  }
  done.forEach((f) => f());
}

/** Clone, mutate, show everywhere now, persist shortly (Rust validates and broadcasts). The
 *  promise resolves once it is saved. */
export function update(mutate: (c: Config) => void): Promise<void> {
  const next: Config = JSON.parse(JSON.stringify(pending ?? store.config));
  mutate(next);
  store.config = next;
  pending = next;
  emit('config:preview', { from: SELF, config: next }).catch(() => {});
  clearTimeout(saveTimer);
  saveTimer = setTimeout(flush, SAVE_AFTER_MS);
  return new Promise((ok) => waiting.push(ok));
}

export function updateWidgetSettings(instance: string, patch: Record<string, unknown>) {
  return update((c) => {
    const w = c.profiles[c.activeProfile].widgets.find((w) => w.instance === instance);
    if (w) w.settings = { ...(w.settings ?? {}), ...patch };
  });
}
