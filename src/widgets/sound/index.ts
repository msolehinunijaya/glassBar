import { Volume2 } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export default defineWidget<{ step: number }>({
  id: 'sound',
  name: 'Sound',
  description: 'Volume, mute, output device, per-app volume',
  icon: Volume2,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { step: 2 },
  Pill,
  Popup: { component: Popup, width: 340, height: 460 },
});
