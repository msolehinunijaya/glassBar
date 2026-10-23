import { SlidersHorizontal } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export default defineWidget<Record<string, never>>({
  id: 'quicksettings',
  name: 'Quick Settings',
  description: 'Wi-Fi, Bluetooth, hotspot, airplane mode, display mode, brightness',
  icon: SlidersHorizontal,
  sizes: ['small'],
  defaultSize: 'small',
  defaults: {},
  Pill,
  Popup: { component: Popup, width: 340, height: 470 },
});
