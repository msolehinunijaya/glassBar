import { AlarmClock } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export default defineWidget<Record<string, never>>({
  id: 'timer',
  name: 'Timer & Stopwatch',
  description: 'Countdown timers with presets, stopwatch with laps',
  icon: AlarmClock,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: {},
  Pill,
  Popup: { component: Popup, width: 340, height: 440 },
});
