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
}

export default defineWidget<ShelfSettings>({
  id: 'shelf',
  name: 'File Shelf',
  description: 'Park a few files by the dock and pick them up later',
  category: 'productivity',
  icon: Archive,
  sizes: ['small', 'medium'],
  defaultSize: 'medium',
  defaults: { files: [], showNames: false },
  Pill,
  Popup: { component: Popup, width: 360, height: 320 },
});
