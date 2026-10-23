import { Wind } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface IpuSettings { station: string; alerts: boolean }

export default defineWidget<IpuSettings>({
  id: 'ipu',
  name: 'IPU (Air Quality)',
  description: 'Air Pollutant Index from the nearest JAS/DOE station',
  icon: Wind,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { station: 'auto', alerts: true },
  Pill,
  Popup: { component: Popup, width: 320, height: 400 },
  SettingsForm,
});
