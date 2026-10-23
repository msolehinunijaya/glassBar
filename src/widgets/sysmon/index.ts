import { Activity } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface SysmonSettings { cpu: boolean; ram: boolean; network: boolean; gpu: boolean; temp: boolean; tempUnit: 'c' | 'f'; tempWarn: string }

export default defineWidget<SysmonSettings>({
  id: 'sysmon',
  name: 'System Monitor',
  description: 'CPU, RAM, GPU and network with live graphs',
  icon: Activity,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: { cpu: true, ram: true, network: false, gpu: false, temp: true, tempUnit: 'c', tempWarn: '80' },
  Pill,
  Popup: { component: Popup, width: 340, height: 460 },
  SettingsForm,
});
