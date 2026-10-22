import { Clock } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface ClockSettings { hour12: boolean; showSeconds: boolean; secondTz: string }

export default defineWidget<ClockSettings>({
  id: 'clock',
  name: 'Clock & Date',
  description: 'Time, date, calendar, optional second time zone',
  icon: Clock,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: { hour12: false, showSeconds: false, secondTz: '' },
  Pill,
  Popup: { component: Popup, width: 320, height: 380 },
  SettingsForm,
});
