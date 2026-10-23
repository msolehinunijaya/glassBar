import { SlidersHorizontal } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';
import type { SoundSettings } from '../sound/settings';

/** trayOnDock: real tray icons inside the Quick Settings pill on the dock; langOnDock: the input language code (EN, MS…). */
/** Battery (was its own widget): section in the panel, icon on the dock after Bluetooth. */
/** Volume (was its own widget): the middle segment on the dock; its options under `sound`. */
export interface QuickSettings { showLanguage: boolean; langOnDock: boolean; showTray: boolean; trayOnDock: boolean; showBattery: boolean; batteryOnDock: boolean; batteryStyle: 'classic' | 'liquid'; soundOnDock: boolean; sound?: Partial<SoundSettings> }

export default defineWidget<QuickSettings>({
  id: 'quicksettings',
  name: 'Quick Settings',
  description: 'Wi-Fi, Bluetooth, volume, battery and power mode, hotspot, airplane mode, display, brightness, keyboard language, system tray icons',
  icon: SlidersHorizontal,
  sizes: ['small'],
  defaultSize: 'small',
  defaults: { showLanguage: true, langOnDock: false, showTray: true, trayOnDock: false, showBattery: true, batteryOnDock: false, batteryStyle: 'classic', soundOnDock: true },
  Pill,
  Popup: { component: Popup, width: 360, height: 680 },
  SettingsForm,
});
