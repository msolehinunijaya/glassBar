import { Hourglass } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export default defineWidget<Record<string, never>>({
  id: 'screentime',
  name: 'Screen Time',
  description: "Today's usage, top apps, 7-day graph (records only while placed)",
  icon: Hourglass,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: {},
  Pill,
  Popup: { component: Popup, width: 340, height: 440 },
});
