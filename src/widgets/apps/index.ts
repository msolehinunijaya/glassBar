import { LayoutGrid } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import SettingsForm from './SettingsForm.svelte';

/** aumid: Store-app identity (Store apps can only be started through it). */
export interface Pin { path: string; args?: string; aumid?: string; /** shown name (web apps: the app, not the browser) */ name?: string }
export interface AppsSettings {
  pinned: Pin[];
  showTitles: boolean;
  /** Start & Search: 'inline' = left of the app icons, 'after' = right of them, 'separate' = their own widget. */
  startPlacement: 'inline' | 'after' | 'separate';
  /** Icons shown before a "+N" chip (0 = all); hovering the launcher shows them all. */
  maxIcons: number;
  searchStyle: 'icon' | 'bar';
  /** The Windows logo on the Start button: blue (as Windows), the accent, the text colour,
   *  the classic four colours, or `startCustom`. */
  startColor?: 'blue' | 'accent' | 'text' | 'classic' | 'custom';
  startCustom?: string;
  /** Icon order chosen by dragging (program keys = lower-case paths). */
  order: string[];
}

export default defineWidget<AppsSettings>({
  id: 'apps',
  name: 'App Launcher',
  description: 'Pinned and running apps, grouped by program',
  category: 'apps',
  icon: LayoutGrid,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'large',
  defaults: { pinned: [], showTitles: false, startPlacement: 'inline', searchStyle: 'icon', order: [], maxIcons: 0 },
  Pill,
  SettingsForm,
});
