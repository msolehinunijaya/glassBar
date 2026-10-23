import { Sparkles } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface AiSettings { model: string; showTokens: boolean }

export default defineWidget<AiSettings>({
  id: 'ai',
  name: 'AI Usage',
  description: 'Ask Claude a quick question; see tokens used today and this month',
  category: 'ai',
  icon: Sparkles,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { model: 'claude-opus-5-5', showTokens: true },
  Pill,
  Popup: { component: Popup, width: 380, height: 520 },
  SettingsForm,
});
