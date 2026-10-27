import { Archive } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export interface ShelfSettings {
  /** Absolute paths, newest first, at most MAX (see shelf.ts). Paths only: the shelf never
   *  moves, copies or deletes a file. */
  files: string[];
  /** Show the file names on the dock, not just the icons. */
  showNames: boolean;
  /** At rest show only the newest file and "+N"; hovering opens the rest. */
  collapse: boolean;
}

export default defineWidget<ShelfSettings>({
  id: 'shelf',
  name: 'File Shelf',
  description: 'Park a few files by the dock and pick them up later',
  category: 'productivity',
  icon: Archive,
  sizes: ['small', 'medium'],
  defaultSize: 'medium',
  defaults: { files: [], showNames: false, collapse: true },
  Pill,
  Popup: { component: Popup, width: 360, height: 320 },
});
