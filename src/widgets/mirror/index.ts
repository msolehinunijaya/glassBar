import { Video } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';
import { DEFAULTS, type MirrorSettings } from './mirror';

export default defineWidget<MirrorSettings>({
  id: 'mirror',
  name: 'Mirror',
  description: 'Your webcam as a mirror: check yourself, take a photo or a short video',
  category: 'media',
  icon: Video,
  sizes: ['small', 'medium'],
  defaultSize: 'medium',
  defaults: DEFAULTS,
  Pill,
  Popup: { component: Popup, width: 360, height: 470 },
  SettingsForm,
});
