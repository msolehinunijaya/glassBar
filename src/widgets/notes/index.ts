import { StickyNote } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface NotesSettings { temporaryDays: number }

export const COLORS: Record<string, string> = {
  yellow: '#ffd60a', green: '#30d158', blue: '#64d2ff', pink: '#ff6482', purple: '#bf5af2', gray: '#98989d',
};

export default defineWidget<NotesSettings>({
  id: 'notes',
  name: 'Quick Note',
  description: 'Sticky notes with pasted text and images',
  icon: StickyNote,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { temporaryDays: 0 },
  Pill,
  Popup: { component: Popup, width: 360, height: 480 },
  SettingsForm,
});
