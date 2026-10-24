// What the Recycle Bin card says, from Windows' answer (or its error). Never a guessed number.
import type { RecycleInfo } from '../../lib/ipc';
import { fmtSize } from '../downloads/format';

export interface BinView { title: string; detail: string; canEmpty: boolean }

export function binView(info: RecycleInfo | null, error: string | null): BinView {
  if (error) return { title: 'N/A', detail: error, canEmpty: false };
  if (!info) return { title: '…', detail: 'Reading the Recycle Bin…', canEmpty: false };
  if (info.items === 0) return { title: '0 B', detail: 'Recycle Bin is empty', canEmpty: false };
  return { title: fmtSize(info.size), detail: `${info.items.toLocaleString()} item${info.items === 1 ? '' : 's'}`, canEmpty: true };
}

/** "would free 4% of C:" style share: the bin's size against the drive's free space. */
export const freeShare = (size: number, free: number) => (free > 0 ? Math.max(1, Math.round((size / free) * 100)) : 0);

/** The confirm text before a permanent delete. */
export const confirmText = (items: number, size: number, drive?: string) =>
  `Permanently delete ${items.toLocaleString()} item${items === 1 ? '' : 's'} (${fmtSize(size)})${drive ? ` from ${drive}` : ''}? This can't be undone.`;
