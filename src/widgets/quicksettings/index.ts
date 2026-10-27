import { SlidersHorizontal } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';
import type { SoundSettings } from '../sound/settings';

/** trayOnDock: real tray icons inside the Quick Settings pill on the dock; langOnDock: the input language code (EN, MS…). */
/** Battery (was its own widget): always the last segment on the dock when the PC has one,
 *  with its own panel. `showBattery` / `batteryOnDock` are older and no longer used. */
/** Volume (was its own widget): the middle segment on the dock; its options under `sound`. */
/** Tiles in the panel, in order; `hiddenTiles` are the ones switched off in Edit tiles. */
export const TILES = [['wifi', 'Wi-Fi'], ['bluetooth', 'Bluetooth'], ['airplane', 'Airplane'], ['hotspot', 'Hotspot'],
  ['nearby', 'Nearby sharing'], ['nightlight', 'Night light'], ['cast', 'Cast']] as const;
export type TileKey = (typeof TILES)[number][0];
/** nearbyMode: the 'on' Nearby sharing mode the tile turns back on (1 my devices, 2 everyone). */
export interface QuickSettings { hiddenTiles?: TileKey[]; nearbyMode?: 1 | 2; showLanguage: boolean; langOnDock: boolean; showTray: boolean; trayOnDock: boolean; showBattery: boolean; batteryOnDock: boolean; batteryStyle: 'classic' | 'liquid' | 'fill'; soundOnDock: boolean; sound?: Partial<SoundSettings> }

export default defineWidget<QuickSettings>({
  id: 'quicksettings',
  name: 'Quick Settings',
  description: 'Wi-Fi, Bluetooth, volume, battery and power mode, hotspot, airplane mode, nearby sharing, night light, cast, display, brightness, keyboard language, system tray icons',
  category: 'connectivity',
  icon: SlidersHorizontal,
  sizes: ['small'],
  defaultSize: 'small',
  defaults: { showLanguage: true, langOnDock: false, showTray: true, trayOnDock: true, showBattery: true, batteryOnDock: false, batteryStyle: 'classic', soundOnDock: true },
  Pill,
  Popup: { component: Popup, width: 360, height: 680 },
  SettingsForm,
});
