import { CloudSun } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface WeatherSettings { unit: 'c' | 'f'; colorful: boolean }

export default defineWidget<WeatherSettings>({
  id: 'weather',
  name: 'Weather',
  description: 'Conditions, feels-like, wind, rain chance, sunrise/sunset, hourly and 3-day forecast',
  icon: CloudSun,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { unit: 'c', colorful: true },
  Pill,
  Popup: { component: Popup, width: 340, height: 640 },
  SettingsForm,
});
