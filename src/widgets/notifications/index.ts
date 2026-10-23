import { Bell } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export default defineWidget<Record<string, never>>({
  id: 'notifications',
  name: 'Notifications',
  description: 'Alert history from timers, prayers, to-dos; Do Not Disturb',
  icon: Bell,
  sizes: ['small'],
  defaultSize: 'small',
  defaults: {},
  Pill,
  Popup: { component: Popup, width: 340, height: 460 },
});
