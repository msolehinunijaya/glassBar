// Pure layout logic for Settings › Widgets: on/off, presets, filtering. No Svelte, no ipc.
import type { WidgetInst, WidgetSize } from '../lib/ipc';
import { categoryOf, type CategoryId } from './categories';

export const isOn = (w: Pick<WidgetInst, 'enabled'>) => w.enabled !== false;

/** What the dock actually shows, in order: widgets that are on, minus the standalone
 *  Start & Search while the App Launcher shows it inline. */
export function visibleOnDock<T extends WidgetInst>(list: T[], known: Record<string, unknown>): T[] {
  const inlineStart = list.some((w) => w.id === 'apps' && isOn(w) && (w.settings?.startPlacement ?? 'inline') !== 'separate');
  return list.filter((w) => known[w.id] && isOn(w) && !(w.id === 'start' && inlineStart));
}

export type LayoutItem = { id: string; size?: WidgetSize };
export type Preset = { id: string; name: string; description: string; items: LayoutItem[] | 'all' };

/** Same as a fresh install (Rust Profile::default). */
export const DEFAULT_LAYOUT: LayoutItem[] = [
  { id: 'start', size: 'small' }, { id: 'apps', size: 'large' }, { id: 'music', size: 'medium' },
  { id: 'quicksettings', size: 'small' }, { id: 'clock', size: 'medium' },
];

export const PRESETS: Preset[] = [
  { id: 'minimal', name: 'Minimal', description: 'Apps, Quick Settings and the clock',
    items: [{ id: 'start' }, { id: 'apps' }, { id: 'quicksettings' }, { id: 'clock' }] },
  { id: 'productivity', name: 'Productivity', description: 'Focus timer, to-dos, notes, timers and progress',
    items: [{ id: 'start' }, { id: 'apps' }, { id: 'productivity' }, { id: 'timer' }, { id: 'progress' }, { id: 'notifications' }, { id: 'quicksettings' }, { id: 'clock' }] },
  { id: 'system', name: 'System Monitoring', description: 'CPU, memory, network, screen time and downloads',
    items: [{ id: 'start' }, { id: 'apps' }, { id: 'sysmon' }, { id: 'screentime' }, { id: 'downloads' }, { id: 'quicksettings' }, { id: 'clock' }] },
  { id: 'media', name: 'Media', description: 'Music, capture and notifications',
    items: [{ id: 'start' }, { id: 'apps' }, { id: 'music' }, { id: 'ocr' }, { id: 'notifications' }, { id: 'quicksettings' }, { id: 'clock' }] },
  { id: 'all', name: 'All Available Widgets', description: 'Every widget on the dock', items: 'all' },
];

type Meta = { defaultSize: WidgetSize; sizes: readonly WidgetSize[] };

/** The listed widgets on, in that order; every other entry kept (settings intact) but off.
 *  Missing widgets are added with defaults. One entry per widget id. */
export function applyLayout(current: WidgetInst[], items: LayoutItem[], known: Record<string, Meta>, now = Date.now()): WidgetInst[] {
  const seen = new Set<string>();
  const on = items.filter((it) => known[it.id] && !seen.has(it.id) && !!seen.add(it.id)).map((it): WidgetInst => {
    const had = current.find((w) => w.id === it.id);
    const size = it.size && known[it.id].sizes.includes(it.size) ? it.size : had?.size ?? known[it.id].defaultSize;
    return had ? { ...had, size, enabled: true } : { id: it.id, instance: `${it.id}-${now.toString(36)}`, size, settings: {}, enabled: true };
  });
  const off = current.filter((w) => !seen.has(w.id)).map((w) => ({ ...w, enabled: false }));
  return [...on, ...off];
}

export const presetItems = (p: Preset, all: string[]): LayoutItem[] => (p.items === 'all' ? all.map((id) => ({ id })) : p.items);

export type Filter = 'all' | 'enabled' | 'disabled' | 'available';

/** Where a widget stands on the active profile. */
export const stateOf = (id: string, placed: WidgetInst[]): Exclude<Filter, 'all'> => {
  const w = placed.find((x) => x.id === id);
  return !w ? 'available' : isOn(w) ? 'enabled' : 'disabled';
};

type Card = { id: string; name: string; description: string; category?: string };
export type Query = { text: string; category: CategoryId | 'all'; filter: Filter };

export function matches(m: Card, placed: WidgetInst[], q: Query) {
  const t = q.text.trim().toLowerCase();
  return (q.category === 'all' || categoryOf(m) === q.category)
    && (q.filter === 'all' || stateOf(m.id, placed) === q.filter)
    && (!t || `${m.name} ${m.description}`.toLowerCase().includes(t));
}

/** Per category: how many widgets, and how many are on. */
export function counts(ms: Card[], placed: WidgetInst[]) {
  const out: Partial<Record<CategoryId, { total: number; on: number }>> = {};
  for (const m of ms) {
    const c = (out[categoryOf(m)] ??= { total: 0, on: 0 });
    c.total++;
    if (stateOf(m.id, placed) === 'enabled') c.on++;
  }
  return out;
}
