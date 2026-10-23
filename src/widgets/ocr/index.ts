import { Aperture } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export type CaptureTab = 'shot' | 'record' | 'ocr' | 'clip';

export interface OcrSettings {
  mode: 'text' | 'code' | 'terminal'; engine: 'windows' | 'ai'; autoCopy: boolean; toast: boolean; hotkey: string;
  /** Last tab opened, and how many days screenshots are kept. */
  tab: CaptureTab; keepDays: number;
  /** Order of the four buttons on the dock; clipboard history on/off. */
  icons: CaptureTab[]; clipHistory: boolean;
}

export default defineWidget<OcrSettings>({
  id: 'ocr',
  name: 'Capture',
  description: 'Screenshot (screen, area, window, scrolling page), screen recording, OCR Snap and clipboard history',
  icon: Aperture,
  sizes: ['medium'],
  defaultSize: 'medium',
  defaults: { mode: 'text', engine: 'windows', autoCopy: true, toast: true, hotkey: 'Super+Shift+T', tab: 'shot', keepDays: 7, icons: ['shot', 'record', 'ocr', 'clip'], clipHistory: true },
  Pill,
  Popup: { component: Popup, width: 380, height: 540 },
  SettingsForm,
});
