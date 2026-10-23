import { BatteryMedium } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export default defineWidget<Record<string, never>>({
  id: 'battery',
  name: 'Battery',
  description: 'Charge level, charging state, time remaining',
  icon: BatteryMedium,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: {},
  Pill,
  Popup: { component: Popup, width: 300, height: 220 },
});
