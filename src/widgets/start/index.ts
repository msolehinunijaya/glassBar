import { Search } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';

export default defineWidget<Record<string, never>>({
  id: 'start',
  name: 'Start & Search',
  description: 'Windows Start menu and Search buttons',
  category: 'apps',
  icon: Search,
  sizes: ['small'],
  defaultSize: 'small',
  defaults: {},
  Pill,
});
