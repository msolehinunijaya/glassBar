import { SlidersHorizontal } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface QuickSettings { showLanguage: boolean; showTray: boolean }

export default defineWidget<QuickSettings>({
  id: 'quicksettings',
  name: 'Quick Settings',
  description: 'Wi-Fi, Bluetooth, hotspot, airplane mode, display, brightness, keyboard language, tray',
  icon: SlidersHorizontal,
  sizes: ['small'],
  defaultSize: 'small',
  defaults: { showLanguage: true, showTray: true },
  Pill,
  Popup: { component: Popup, width: 340, height: 600 },
  SettingsForm,
});
