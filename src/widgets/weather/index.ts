import { CloudSun } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface WeatherSettings { unit: 'c' | 'f' }

export default defineWidget<WeatherSettings>({
  id: 'weather',
  name: 'Weather',
  description: 'Current conditions, hourly and 3-day forecast',
  icon: CloudSun,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { unit: 'c' },
  Pill,
  Popup: { component: Popup, width: 340, height: 420 },
  SettingsForm,
});
