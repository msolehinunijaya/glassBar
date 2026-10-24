import { Bell } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface NotifSettings { windowsTab: boolean; badge: boolean; rememberTab: boolean; lastTab: 'dock' | 'windows' }

export default defineWidget<NotifSettings>({
  id: 'notifications',
  name: 'Notifications',
  description: 'Alert history from timers, prayers, to-dos; Do Not Disturb',
  category: 'media',
  icon: Bell,
  sizes: ['small'],
  defaultSize: 'small',
  defaults: { windowsTab: true, badge: true, rememberTab: true, lastTab: 'dock' },
  Pill,
  Popup: { component: Popup, width: 340, height: 480 },
  SettingsForm,
});
