import { Music } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import { MUSIC_DEFAULTS, type MusicSettings } from './settings';

export type { ArtShape, MusicLayout, MusicParts, MusicSettings, MusicStyle, MusicViz } from './settings';

export default defineWidget<MusicSettings>({
  id: 'music',
  name: 'Music',
  description: 'Now playing from Spotify, browsers and any media app',
  category: 'media',
  icon: Music,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: MUSIC_DEFAULTS,
  Pill,
  Popup: { component: Popup, width: 320, height: 560 },
  perf: { low: 'No visualizer, still playing badge, no karaoke wipe', normal: 'Visualizer up to 30 fps', high: 'Visualizer at your frame rate' },
});
