// Pure helpers for the Recycle Bin widget (tested in trash.test.ts).
import type { RecycleInfo, TrashResult } from '../../lib/ipc';
import { nameOf } from '../shelf/shelf';

const items = (n: number) => `${n} item${n === 1 ? '' : 's'}`;

/** What the card after a drop says. */
export function dropSummary(r: TrashResult): string {
  const bad = r.failed.length ? `${r.failed.length} couldn't be moved (${nameOf(r.failed[0][0])}: ${r.failed[0][1]})` : '';
  if (!r.moved) return bad || 'Nothing was moved';
  return bad ? `Moved ${items(r.moved)} · ${bad}` : `Moved ${items(r.moved)} to the Recycle Bin`;
}

/** "Documents › Work": the last two folders of where an item was deleted from. */
export function fromLabel(folder: string): string {
  const parts = folder.split(/[\\/]+/).filter(Boolean);
  return parts.slice(-2).join(' › ');
}

/** The dock icon's state: no answer yet (or Windows refused) = unknown, never a guess. */
export const binLook = (info: RecycleInfo | null): 'unknown' | 'empty' | 'full' => (!info ? 'unknown' : info.items > 0 ? 'full' : 'empty');
