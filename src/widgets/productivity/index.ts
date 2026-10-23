import { ListChecks } from '@lucide/svelte';
import type { PomoCfg } from '../../lib/ipc';
import type { NotesSettings } from '../notes/index';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import SettingsForm from './SettingsForm.svelte';

/** Which parts show on the dock, plus the parts' own settings (flat). */
export interface ProductivitySettings extends PomoCfg, NotesSettings { showPomo: boolean; showTodo: boolean; showNotes: boolean }

export default defineWidget<ProductivitySettings>({
  id: 'productivity',
  name: 'Pomodoro · To-Do · Notes',
  description: 'Focus timer, to-dos and sticky notes in one widget; pick which show on the dock',
  category: 'productivity',
  icon: ListChecks,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: { showPomo: true, showTodo: true, showNotes: true, workMin: 25, shortMin: 5, longMin: 15, every: 4, temporaryDays: 0 },
  Pill,
  SettingsForm,
});
