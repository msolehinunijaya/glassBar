import { AudioLines } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';
import { EQ_DEFAULTS, type EqSettings } from './settings';

export type { EqSettings } from './settings';

export default defineWidget<EqSettings>({
  id: 'equalizer',
  name: 'Equalizer',
  description: 'Live spectrum of whatever your PC is playing: bars, LED, neon, waves, ring and more',
  category: 'media',
  icon: AudioLines,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: EQ_DEFAULTS,
  Pill,
  Popup: { component: Popup, width: 360, height: 620 },
  SettingsForm,
  perf: { low: 'Up to 15 fps, line when silent', normal: 'Up to 30 fps', high: 'Your frame rate (up to 60 fps)' },
});
