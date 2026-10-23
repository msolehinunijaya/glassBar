import { Music } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';

export interface MusicSettings {
  showControls: boolean; style: 'player' | 'classic' | 'mini'; showProgress: boolean;
  /** Live level visualizer in the Player design, and its width in px (adjustable). */
  showViz: boolean; vizWidth: number;
}

export default defineWidget<MusicSettings>({
  id: 'music',
  name: 'Music',
  description: 'Now playing from Spotify, browsers and any media app',
  icon: Music,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: { showControls: true, style: 'player', showProgress: true, showViz: true, vizWidth: 64 },
  Pill,
  Popup: { component: Popup, width: 320, height: 560 },
});
