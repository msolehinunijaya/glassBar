import { ScanText } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface OcrSettings { mode: 'text' | 'code' | 'terminal'; engine: 'windows' | 'ai'; autoCopy: boolean; toast: boolean; hotkey: string }

export default defineWidget<OcrSettings>({
  id: 'ocr',
  name: 'OCR Snap',
  description: 'Grab text from anywhere on screen (Win+Shift+T)',
  icon: ScanText,
  sizes: ['small'],
  defaultSize: 'small',
  defaults: { mode: 'text', engine: 'windows', autoCopy: true, toast: true, hotkey: 'Super+Shift+T' },
  Pill,
  Popup: { component: Popup, width: 360, height: 480 },
  SettingsForm,
});
