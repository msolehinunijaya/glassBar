import { Hourglass } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';
import type { Mode } from './period';

export interface ProgressSettings { mode: Mode; showTimes: boolean }

export default defineWidget<ProgressSettings>({
  id: 'progress',
  name: 'Time Progress',
  description: 'How much of the day, month or year has passed',
  category: 'productivity',
  icon: Hourglass,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { mode: 'day', showTimes: true },
  Pill,
  Popup: { component: Popup, width: 262, height: 372 },
  SettingsForm,
});
