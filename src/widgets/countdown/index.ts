import { Hourglass } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';
import type { Countdown } from './dates';

/** items: the countdowns (small, made by the user, so they live in config and travel with an
 *  export). pinned: the one on the dock (null = the nearest). notify: a notification on the day. */
export interface CountdownSettings { items: Countdown[]; pinned: string | null; notify: boolean }

export default defineWidget<CountdownSettings>({
  id: 'countdown',
  name: 'Countdowns',
  description: 'Days to Raya, Merdeka, a deadline or any date',
  category: 'productivity',
  icon: Hourglass,
  sizes: ['small', 'medium'],
  defaultSize: 'medium',
  defaults: { items: [], pinned: null, notify: true },
  Pill,
  Popup: { component: Popup, width: 340, height: 480 },
  SettingsForm,
});
