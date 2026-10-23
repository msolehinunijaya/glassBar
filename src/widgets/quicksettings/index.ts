import { SlidersHorizontal } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

/** trayOnDock: real tray icons inside the Quick Settings pill on the dock. */
export interface QuickSettings { showLanguage: boolean; showTray: boolean; trayOnDock: boolean }

export default defineWidget<QuickSettings>({
  id: 'quicksettings',
  name: 'Quick Settings',
  description: 'Wi-Fi, Bluetooth, hotspot, airplane mode, display, brightness, keyboard language, system tray icons',
  icon: SlidersHorizontal,
  sizes: ['small'],
  defaultSize: 'small',
  defaults: { showLanguage: true, showTray: true, trayOnDock: false },
  Pill,
  Popup: { component: Popup, width: 360, height: 680 },
  SettingsForm,
});
