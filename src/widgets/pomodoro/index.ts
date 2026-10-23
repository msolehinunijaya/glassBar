import { Timer } from '@lucide/svelte';
import type { PomoCfg } from '../../lib/ipc';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export const PHASE = {
  work: { name: 'Focus', color: '#ff6b5b' },
  short: { name: 'Short break', color: '#30d158' },
  long: { name: 'Long break', color: '#64d2ff' },
} as const;

export default defineWidget<PomoCfg>({
  id: 'pomodoro',
  name: 'Pomodoro',
  description: 'Focus sessions with short and long breaks',
  icon: Timer,
  sizes: ['small', 'medium'],
  defaultSize: 'medium',
  defaults: { workMin: 25, shortMin: 5, longMin: 15, every: 4, autoStart: true },
  Pill,
  Popup: { component: Popup, width: 320, height: 360 },
  SettingsForm,
});
