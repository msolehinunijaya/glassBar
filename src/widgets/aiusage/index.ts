import { Gauge } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface AiUsageSettings {
  /** Banners at 80 % / 95 % of a limit (read by Rust, aiusage.rs). */
  alertsClaude: boolean;
  alertsChatgpt: boolean;
  /** Banner when a limit that was past 80 % resets. */
  alertReset: boolean;
  tab: 'claude' | 'chatgpt' | 'all';
  /** Usage source per provider when it has more than one (e.g. Claude: local logs or Admin API). */
  prefer: { claude?: string; chatgpt?: string };
}

export default defineWidget<AiUsageSettings>({
  id: 'aiusage',
  name: 'AI Usage',
  description: 'How much of your Claude and ChatGPT limits you have used',
  category: 'ai',
  icon: Gauge,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: { alertsClaude: true, alertsChatgpt: true, alertReset: true, tab: 'claude', prefer: {} },
  Pill,
  Popup: { component: Popup, width: 400, height: 720 },
  SettingsForm,
});
