import { BatteryMedium } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export interface BatterySettings { style: 'classic' | 'liquid' }

export default defineWidget<BatterySettings>({
  id: 'battery',
  name: 'Battery',
  description: 'Charge level (classic or liquid style), power mode',
  icon: BatteryMedium,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { style: 'classic' },
  Pill,
  Popup: { component: Popup, width: 320, height: 420 },
});
