import { Hourglass } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

/** showGraph: a small 7-day bar graph beside the time on the dock. */
export interface ScreenTimeSettings { showGraph: boolean }

export default defineWidget<ScreenTimeSettings>({
  id: 'screentime',
  name: 'Screen Time',
  description: "Today's usage, top apps, 7-day graph (records only while placed)",
  icon: Hourglass,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { showGraph: true },
  Pill,
  Popup: { component: Popup, width: 340, height: 440 },
});
