import { ChevronUp } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';

export default defineWidget<Record<string, never>>({
  id: 'tray',
  name: 'System Tray',
  description: "Opens Windows' hidden tray icons (click) or tray settings (right-click)",
  icon: ChevronUp,
  sizes: ['small'],
  defaultSize: 'small',
  defaults: {},
  Pill,
});
