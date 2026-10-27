import { AlarmClock } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

/** progress: fill behind the time · drain: show time remaining instead of elapsed ·
 *  warn: orange in the final minute · shimmer: soft sweep while the stopwatch runs. */
export interface TimerSettings { progress: boolean; drain: boolean; warn: boolean; shimmer: boolean }

export default defineWidget<TimerSettings>({
  id: 'timer',
  name: 'Timer & Stopwatch',
  description: 'Countdown timers with presets, stopwatch with laps',
  category: 'productivity',
  icon: AlarmClock,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { progress: true, drain: false, warn: true, shimmer: true },
  Pill,
  Popup: { component: Popup, width: 340, height: 440 },
  SettingsForm,
  perf: { low: 'No stopwatch shimmer' },
});
