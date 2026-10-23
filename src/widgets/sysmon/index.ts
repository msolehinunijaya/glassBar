import { Activity } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface SysmonSettings { gpu: boolean; network: boolean }

export default defineWidget<SysmonSettings>({
  id: 'sysmon',
  name: 'System Monitor',
  description: 'CPU, RAM, GPU and network with live graphs',
  icon: Activity,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: { gpu: false, network: true },
  Pill,
  Popup: { component: Popup, width: 340, height: 380 },
  SettingsForm,
});
