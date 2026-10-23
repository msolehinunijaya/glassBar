import { ChevronUp } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface TraySettings { native: boolean; showPromoted: boolean; showOverflow: boolean; compact: boolean }

export default defineWidget<TraySettings>({
  id: 'tray',
  name: 'System Tray',
  description: 'Tray icons of background apps: on the dock and in a panel',
  icon: ChevronUp,
  sizes: ['small'],
  defaultSize: 'small',
  defaults: { native: true, showPromoted: true, showOverflow: true, compact: false },
  Pill,
  Popup: { component: Popup, width: 340, height: 300 },
  SettingsForm,
});
