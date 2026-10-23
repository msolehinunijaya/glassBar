import { HardDrive } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

/** drives: mount points ("C:\\"); empty = system drive. Thresholds are used-% strings. */
export interface DiskSettings { drives: string[]; high: string; critical: string; showFree: boolean }

export default defineWidget<DiskSettings>({
  id: 'disk',
  name: 'Disk Storage',
  description: 'Used and free space on your drives',
  icon: HardDrive,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { drives: [], high: '80', critical: '90', showFree: false },
  Pill,
  Popup: { component: Popup, width: 340, height: 380 },
  SettingsForm,
});
