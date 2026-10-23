import { Clock } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';
import type { Zone } from './zones';

export interface ClockSettings {
  hour12: boolean;
  showSeconds: boolean;
  /** standard pill · prominent (bigger, bolder, accent panel) · popup (hover shows a large clock) */
  appearance: 'standard' | 'prominent' | 'popup';
  /** World clocks, in the user's order. */
  zones: Zone[];
  /** Older single "second time zone" setting; folded into `zones` when read. */
  secondTz: string;
}

export default defineWidget<ClockSettings>({
  id: 'clock',
  name: 'Clock & Date',
  description: 'Time and date, calendar, world clocks',
  icon: Clock,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: { hour12: false, showSeconds: false, appearance: 'standard', zones: [], secondTz: '' },
  Pill,
  Popup: { component: Popup, width: 340, height: 560 },
  SettingsForm,
});
