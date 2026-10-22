import { LayoutGrid } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface Pin { path: string; args?: string }
export interface AppsSettings { pinned: Pin[]; showTitles: boolean }

export default defineWidget<AppsSettings>({
  id: 'apps',
  name: 'App Launcher',
  description: 'Pinned and running apps, grouped by program',
  icon: LayoutGrid,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'large',
  defaults: { pinned: [], showTitles: false },
  Pill,
  SettingsForm,
});
