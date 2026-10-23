import { LayoutGrid } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface Pin { path: string; args?: string }
export interface AppsSettings {
  pinned: Pin[];
  showTitles: boolean;
  /** Start & Search: 'inline' = right before the app icons, 'separate' = their own widget. */
  startPlacement: 'inline' | 'separate';
  searchStyle: 'icon' | 'bar';
}

export default defineWidget<AppsSettings>({
  id: 'apps',
  name: 'App Launcher',
  description: 'Pinned and running apps, grouped by program',
  icon: LayoutGrid,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'large',
  defaults: { pinned: [], showTitles: false, startPlacement: 'inline', searchStyle: 'icon' },
  Pill,
  SettingsForm,
});
