import { Wind } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface City { name: string; lat: number; lon: number }
export interface Fav { label: string; country: 'MY' | 'other'; station?: string; city?: City }

export interface IpuSettings {
  /** 'auto' = my location · 'MY' = browse Malaysia (state → station) · 'other' = a city abroad (AQI US) */
  country: 'auto' | 'MY' | 'other';
  station: string;
  city: City | null;
  favorites: Fav[];
  alerts: boolean;
}

export default defineWidget<IpuSettings>({
  id: 'ipu',
  name: 'IPU (Air Quality)',
  description: 'Air Pollutant Index: nearest JAS/DOE station, or browse by state/station',
  icon: Wind,
  sizes: ['small', 'medium'],
  defaultSize: 'small',
  defaults: { country: 'auto', station: 'auto', city: null, favorites: [], alerts: true },
  Pill,
  Popup: { component: Popup, width: 340, height: 560 },
  SettingsForm,
});
