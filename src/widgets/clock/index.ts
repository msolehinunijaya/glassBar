import { Clock } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';
import type { Zone } from './zones';
import type { AmPm, DateOpts, DateStyle } from './format';

export interface ClockSettings {
  /** 'system' follows Windows' regional setting (24-hour if unknown). Unset in older configs:
   *  then the old `hour12` switch decides. */
  timeFormat?: 'system' | '24' | '12';
  /** Older 12/24-hour switch, still read when `timeFormat` is unset. */
  hour12: boolean;
  ampm: AmPm;
  ampmSize: 'small' | 'same' | 'hidden';
  /** 'auto' = 02:05 in 24-hour, 2:05 in 12-hour. */
  leadingZero: 'auto' | 'on' | 'off';
  showSeconds: boolean;
  worldSeconds: boolean;
  dateStyle: DateStyle;
  dateSep: DateOpts['sep'];
  weekday: DateOpts['weekday'];
  datePattern: string;
  /** What the pill shows (the small size always shows the time only). */
  show: 'time' | 'date' | 'both';
  layout: 'inline' | 'stacked';
  /** Sun / moon next to each world clock. */
  dayNight: boolean;
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
  description: 'Time and date in your format, calendar, world clocks with day/night',
  icon: Clock,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: {
    hour12: false, ampm: 'PM', ampmSize: 'small', leadingZero: 'auto', showSeconds: false, worldSeconds: false,
    dateStyle: 'weekday', dateSep: '/', weekday: 'none', datePattern: 'ddd, D MMM', show: 'both', layout: 'inline',
    dayNight: true, appearance: 'standard', zones: [], secondTz: '',
  },
  Pill,
  Popup: { component: Popup, width: 340, height: 560 },
  SettingsForm,
});
