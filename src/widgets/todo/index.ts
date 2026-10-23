import { ListChecks } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export default defineWidget<Record<string, never>>({
  id: 'todo',
  name: 'To-Do',
  description: 'Quick tasks with due-date reminders',
  icon: ListChecks,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: {},
  Pill,
  Popup: { component: Popup, width: 340, height: 460 },
});
