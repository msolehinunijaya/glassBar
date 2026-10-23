import { Moon } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface SolatSettings { zone: string; showImsak: boolean; alerts: boolean; azanFile: string }

export default defineWidget<SolatSettings>({
  id: 'solat',
  name: 'Waktu Solat',
  description: 'Prayer times (JAKIM zones, worldwide fallback), countdown, alerts',
  icon: Moon,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: { zone: 'auto', showImsak: false, alerts: true, azanFile: '' },
  Pill,
  Popup: { component: Popup, width: 320, height: 420 },
  SettingsForm,
});
