import { Music } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export interface MusicSettings { showControls: boolean; style: 'classic' | 'mini'; showProgress: boolean }

export default defineWidget<MusicSettings>({
  id: 'music',
  name: 'Music',
  description: 'Now playing from Spotify, browsers and any media app',
  icon: Music,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: { showControls: true, style: 'classic', showProgress: true },
  Pill,
  Popup: { component: Popup, width: 320, height: 470 },
});
