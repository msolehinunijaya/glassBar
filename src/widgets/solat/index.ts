import { Moon } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

/** azanSound: what plays at prayer time: the opening of the adzan, all of it, or the normal chime.
 *  pauseMusic: pause other playing media during the adzan and resume it afterwards. */
export interface SolatSettings { zone: string; showImsak: boolean; alerts: boolean; azanSound: 'short' | 'full' | 'chime'; azanFile: string; pauseMusic: boolean; /** Adzan volume, 0-100 %. */ azanVolume: number; ackKey: string }

export default defineWidget<SolatSettings>({
  id: 'solat',
  name: 'Waktu Solat',
  description: 'Prayer times (JAKIM zones, worldwide fallback), countdown, alerts',
  category: 'info',
  icon: Moon,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: { zone: 'auto', showImsak: false, alerts: true, azanSound: 'short', azanFile: '', pauseMusic: true, azanVolume: 100, ackKey: '' },
  Pill,
  Popup: { component: Popup, width: 320, height: 420 },
  SettingsForm,
});
