import { Volume2 } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export interface SoundSettings { step: number; showSlider: boolean }

export default defineWidget<SoundSettings>({
  id: 'sound',
  name: 'Sound',
  description: 'Volume, mute, output device, per-app volume',
  icon: Volume2,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { step: 2, showSlider: false },
  Pill,
  Popup: { component: Popup, width: 340, height: 480 },
});
