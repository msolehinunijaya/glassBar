import { Shield } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';
import { PRIVACY_DEFAULTS, type PrivacySettings } from './settings';

export type { PrivacySettings } from './settings';

export default defineWidget<PrivacySettings>({
  id: 'privacy',
  name: 'Privacy Monitor',
  description: 'Which apps use your camera, microphone, location or screen right now, with the privacy and device controls',
  category: 'system',
  icon: Shield,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: PRIVACY_DEFAULTS,
  Pill,
  Popup: { component: Popup, width: 380, height: 620 },
  SettingsForm,
});
