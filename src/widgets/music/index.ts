import { Music } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export default defineWidget<{ showControls: boolean }>({
  id: 'music',
  name: 'Music',
  description: 'Now playing from Spotify, browsers and any media app',
  icon: Music,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: { showControls: true },
  Pill,
  Popup: { component: Popup, width: 320, height: 420 },
});
