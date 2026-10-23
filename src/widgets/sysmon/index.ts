import { Activity } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface SysmonSettings {
  cpu: boolean; ram: boolean; network: boolean; disk: boolean; gpu: boolean;
  temp: boolean; tempUnit: 'c' | 'f'; tempWarn: string;
  /** Drives on the bar (mount points); empty = system drive. Thresholds are used-% strings. */
  drives: string[]; diskHigh: string; diskCritical: string;
}

export default defineWidget<SysmonSettings>({
  id: 'sysmon',
  name: 'System Monitor',
  description: 'CPU, RAM, network and disk storage with live graphs',
  icon: Activity,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: { cpu: true, ram: true, network: false, disk: false, gpu: false, temp: true, tempUnit: 'c', tempWarn: '80', drives: [], diskHigh: '80', diskCritical: '90' },
  Pill,
  Popup: { component: Popup, width: 360, height: 640 },
  SettingsForm,
});
