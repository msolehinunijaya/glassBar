import { Aperture } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export type CaptureTab = 'shot' | 'record' | 'ocr' | 'clip' | 'color';

export interface OcrSettings {
  mode: 'text' | 'code' | 'terminal'; engine: 'windows'; autoCopy: boolean; toast: boolean; hotkey: string;
  /** Last tab opened, and how many days screenshots are kept. */
  tab: CaptureTab; keepDays: number;
  /** Order of the four buttons on the dock; clipboard history on/off. */
  icons: CaptureTab[]; clipHistory: boolean;
  /** Look of the four icons (dock and panel tabs). */
  iconStyle: IconStyle;
  /** Colour picker: its hotkey, what is copied, and the last colours picked (hex, newest first). */
  colorHotkey: string; colorFormat: import('./color').ColorFormat; colors: string[];
}
/** 'tile': white symbol on a colour tile · 'plain': simple symbol · 'outline': symbol in its colour. */
export type IconStyle = 'tile' | 'plain' | 'outline';

export default defineWidget<OcrSettings>({
  id: 'ocr',
  name: 'Capture',
  description: 'Screenshot (screen, area, window, scrolling page), screen recording, OCR Snap, colour picker and clipboard history',
  category: 'apps',
  icon: Aperture,
  sizes: ['medium'],
  defaultSize: 'medium',
  defaults: { mode: 'text', engine: 'windows', autoCopy: true, toast: true, hotkey: 'Super+Shift+T', tab: 'shot', keepDays: 7, icons: ['shot', 'record', 'ocr', 'clip', 'color'], clipHistory: true, iconStyle: 'tile', colorHotkey: 'Super+Shift+C', colorFormat: 'hex', colors: [] },
  Pill,
  Popup: { component: Popup, width: 380, height: 540 },
  SettingsForm,
});
