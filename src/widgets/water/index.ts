import { Droplet } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import { WATER_DEFAULTS, type WaterSettings } from './settings';

export type { WaterSettings } from './settings';

export default defineWidget<WaterSettings>({
  id: 'water',
  name: 'Water',
  description: 'Daily water goal with reminders on the dock',
  category: 'productivity',
  icon: Droplet,
  sizes: ['small', 'medium'],
  defaultSize: 'medium',
  defaults: WATER_DEFAULTS,
  Pill,
  Popup: { component: Popup, width: 340, height: 520 },
});
