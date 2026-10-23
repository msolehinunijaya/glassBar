import { Download } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

/** limit: rows in the panel · newDot: "new" dot until hovered/opened · subfolders: look
 *  inside folders too · folder: '' = the Downloads folder · toast: alert when one finishes. */
export interface DownloadsSettings { limit: 5 | 10 | 20; newDot: boolean; subfolders: boolean; folder: string; toast: boolean }

export default defineWidget<DownloadsSettings>({
  id: 'downloads',
  name: 'Downloads',
  description: 'Your latest downloads: open with a click, show in folder, drag into other apps',
  icon: Download,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: { limit: 10, newDot: true, subfolders: false, folder: '', toast: false },
  Pill,
  Popup: { component: Popup, width: 360, height: 520 },
  SettingsForm,
});
