// Settings › Performance: copy and the small pure helpers behind the page.
import type { PerfMode, PerfSample } from '../lib/ipc';

type ModeText = Partial<Record<PerfMode, string>>;

export const MODE_CARDS: { mode: PerfMode; title: string; summary: string }[] = [
  { mode: 'low', title: 'Low', summary: 'Lightest. Things move only when they must.' },
  { mode: 'normal', title: 'Normal', summary: 'Smooth where it counts, calm when idle.' },
  { mode: 'high', title: 'High', summary: 'Everything at full smoothness.' },
];

/** Changes that belong to GlassBar itself rather than one widget. */
export const APP_ROWS: { name: string; perf: ModeText }[] = [
  { name: 'Popup & banner', perf: { low: 'Memory freed when hidden; opens a moment slower' } },
  { name: 'Whole bar', perf: {
    normal: 'Animations pause while auto-hidden',
    low: 'Animations pause while auto-hidden',
    high: 'Animations keep running while auto-hidden',
  } },
];

/** What `mode` changes, row by row in the given order, and who it leaves alone (A–Z). */
export function changeList(mode: PerfMode, items: { name: string; perf?: ModeText }[]) {
  const rows = items.filter((i) => i.perf?.[mode]).map((i) => ({ name: i.name, text: i.perf![mode]! }));
  const unchanged = items.filter((i) => !i.perf?.[mode]).map((i) => i.name).sort((a, b) => a.localeCompare(b));
  return { rows, unchanged };
}

/** Bytes downloaded across the samples (they cover the last hour at most). */
export function netLastHour(s: PerfSample[]): number {
  return s.length < 2 ? 0 : s[s.length - 1].netBytes - s[0].netBytes;
}

export function fmtBytes(n: number): string {
  if (n < 1048576) return `${Math.round(n / 1024)} KB`;
  if (n < 1073741824) return `${(n / 1048576).toFixed(1)} MB`;
  return `${(n / 1073741824).toFixed(1)} GB`;
}

export const fmtPct = (v: number | null): string => (v === null ? '—' : `${v.toFixed(1)}%`);
