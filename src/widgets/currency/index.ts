import { ArrowLeftRight } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

/** favorites: "USD>MYR" pairs. */
export interface CurrencySettings { from: string; to: string; amount: number; favorites: string[] }

export default defineWidget<CurrencySettings>({
  id: 'currency',
  name: 'Currency Converter',
  description: 'Live exchange rates (indicative, not bank rates)',
  icon: ArrowLeftRight,
  sizes: ['small', 'medium'],
  defaultSize: 'medium',
  defaults: { from: 'USD', to: 'MYR', amount: 1, favorites: [] },
  Pill,
  Popup: { component: Popup, width: 340, height: 440 },
  SettingsForm,
});
