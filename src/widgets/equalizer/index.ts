import { AudioLines } from '@lucide/svelte';
import { defineWidget } from '../types';
import Pill from './Pill.svelte';
import Popup from './Popup.svelte';
import SettingsForm from './SettingsForm.svelte';

export interface EqSettings {
  style: 'bars' | 'mirror' | 'glow';
  /** Size: length along the bar (px), share of the bar's thickness used (%), bars, gap and corner (px). */
  length: number; fill: number; bands: number; gap: number; radius: number;
  /** Colour. hueSpeed (°/s) slowly cycles rainbow colours. */
  colorMode: 'accent' | 'solid' | 'gradient' | 'rainbow';
  color1: string; color2: string; direction: 'across' | 'up'; hueSpeed: number;
  glow: boolean; glowSize: number; glowStrength: number; glowColor: 'match' | 'custom'; glowCustom: string;
  /** Opacity, %. */
  barOpacity: number; bgOpacity: number; idleOpacity: number;
  /** Motion: gain %, fall speed 1–10, peak caps and how long they hang (ms), right-to-left. */
  gain: number; fall: number; peaks: boolean; peakHold: number; flip: boolean;
  /** Frequency range (Hz, strings for the choice buttons), bass/treble tilt −100..100, spacing. */
  lowHz: string; highHz: string; tilt: number; spacing: 'log' | 'linear';
  idle: 'dots' | 'line' | 'hide';
  fps: '30' | '60';
}

export const EQ_DEFAULTS: EqSettings = {
  style: 'bars', length: 120, fill: 80, bands: 24, gap: 2, radius: 2,
  colorMode: 'rainbow', color1: '#00e5ff', color2: '#ff2bd6', direction: 'across', hueSpeed: 0,
  glow: true, glowSize: 6, glowStrength: 60, glowColor: 'match', glowCustom: '#ffffff',
  barOpacity: 100, bgOpacity: 0, idleOpacity: 50,
  gain: 100, fall: 4, peaks: true, peakHold: 600, flip: false,
  lowHz: '30', highHz: '16000', tilt: 30, spacing: 'log',
  idle: 'dots', fps: '60',
};

export default defineWidget<EqSettings>({
  id: 'equalizer',
  name: 'Equalizer',
  description: 'Live spectrum of whatever your PC is playing, in bars, mirror or neon glow',
  category: 'media',
  icon: AudioLines,
  sizes: ['small', 'medium', 'large'],
  defaultSize: 'medium',
  defaults: EQ_DEFAULTS,
  Pill,
  Popup: { component: Popup, width: 360, height: 620 },
  SettingsForm,
});
