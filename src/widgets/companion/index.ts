import { Smile } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import { COMPANION_DEFAULTS, type CompanionSettings } from './settings';

export type { CompanionSettings } from './settings';

export default defineWidget<CompanionSettings>({
  id: 'companion',
  name: 'Companion',
  description: 'A small creature on the dock that blinks, can be tickled, and reacts to what you are doing',
  category: 'other',
  icon: Smile,
  sizes: ['small', 'medium'],
  defaultSize: 'medium',
  defaults: COMPANION_DEFAULTS,
  Pill,
  Popup: { component: Popup, width: 360, height: 420 },
});
