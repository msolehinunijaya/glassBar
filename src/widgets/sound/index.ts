import { Volume2 } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export type SoundLayout = 'icon' | 'horizontal' | 'vertical';
/** `showSlider` is the older on/off setting; kept so existing bars keep their slider. */
export interface SoundSettings { step: number; layout: SoundLayout; wheel: boolean; showSlider?: boolean }

export const layoutOf = (s: SoundSettings): SoundLayout => (s.layout === 'icon' && s.showSlider ? 'horizontal' : s.layout);

export default defineWidget<SoundSettings>({
  id: 'sound',
  name: 'Sound',
  description: 'Volume (icon, horizontal or vertical slider), output device, per-app volume',
  icon: Volume2,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { step: 2, layout: 'icon', wheel: false },
  Pill,
  Popup: { component: Popup, width: 340, height: 520 },
});
