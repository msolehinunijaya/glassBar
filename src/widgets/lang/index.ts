import { Languages } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export default defineWidget<Record<string, never>>({
  id: 'lang',
  name: 'Keyboard Language',
  description: 'Shows the input language (EN, MS, …); click to switch',
  icon: Languages,
  sizes: ['small'],
  defaultSize: 'small',
  defaults: {},
  Pill,
  Popup: { component: Popup, width: 280, height: 260 },
});
