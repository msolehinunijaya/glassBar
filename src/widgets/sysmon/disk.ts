import type { DiskInfo } from '../../lib/ipc';

export type Level = 'normal' | 'high' | 'critical';

export const usedPct = (d: DiskInfo) => (d.total ? Math.round(((d.total - d.available) / d.total) * 100) : 0);

export const level = (pct: number, high: number, critical: number): Level =>
  pct >= critical ? 'critical' : pct >= high ? 'high' : 'normal';

export const levelColor: Record<Level, string> = { normal: 'var(--accent)', high: '#ff9f0a', critical: '#ff453a' };

export function bytes(n: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let i = 0;
  while (n >= 1024 && i < units.length - 1) { n /= 1024; i++; }
  return `${n >= 100 || Number.isInteger(n) ? Math.round(n) : n.toFixed(1)} ${units[i]}`;
}

export const letter = (mount: string) => mount.replace(/\\$/, '');

/** Selected drives that are present; with nothing selected, C: (or the first drive). */
export function pickDrives(all: DiskInfo[], picked: string[]): DiskInfo[] {
  const sel = all.filter((d) => picked.includes(d.mount));
  if (sel.length) return sel;
  const c = all.find((d) => d.mount.toUpperCase().startsWith('C:'));
  return c ? [c] : all.slice(0, 1);
}
