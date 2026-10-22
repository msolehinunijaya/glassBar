// SDK template: copy this folder to src/widgets/<your-id>/, rename, edit.
// Folders starting with "_" only appear in dev builds, so this stays out of releases.
import { Sparkles } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface HelloSettings { name: string }

export default defineWidget<HelloSettings>({
  id: 'hello',                         // unique, stored in config
  name: 'Hello (template)',            // shown in the gallery
  description: 'Starter widget from the SDK template',
  icon: Sparkles,
  sizes: ['small', 'medium'],          // sizes the user can cycle through in edit mode
  defaultSize: 'medium',
  defaults: { name: 'world' },         // merged under the saved settings
  Pill,                                // what sits in the bar
  Popup: { component: Popup, width: 300, height: 220 }, // optional panel, size in px
  SettingsForm,                        // optional form in the Settings window
});
