import { Aperture } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface OcrSettings {
  mode: 'text' | 'code' | 'terminal'; engine: 'windows' | 'ai'; autoCopy: boolean; toast: boolean; hotkey: string;
  /** Last tab opened, and how many days screenshots are kept. */
  tab: 'shot' | 'record' | 'ocr'; keepDays: number;
}

export default defineWidget<OcrSettings>({
  id: 'ocr',
  name: 'Capture',
  description: 'Screenshot (screen, area, window, scrolling page), screen recording and OCR Snap',
  icon: Aperture,
  sizes: ['small'],
  defaultSize: 'small',
  defaults: { mode: 'text', engine: 'windows', autoCopy: true, toast: true, hotkey: 'Super+Shift+T', tab: 'shot', keepDays: 7 },
  Pill,
  Popup: { component: Popup, width: 380, height: 540 },
  SettingsForm,
});
