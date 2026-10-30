import { Trash2 } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

/** lastDrop: the files just dropped on the dock icon, for the panel's Undo (a minute). */
export interface TrashSettings { lastDrop?: { at: number; paths: string[]; text: string; ok: boolean } | null }

export default defineWidget<TrashSettings>({
  id: 'trash',
  name: 'Recycle Bin',
  description: 'Full or empty at a glance, drop files on it, restore or empty',
  category: 'system',
  icon: Trash2,
  sizes: ['small', 'medium'],
  defaultSize: 'medium',
  defaults: { lastDrop: null },
  Pill,
  Popup: { component: Popup, width: 380, height: 520 },
});
