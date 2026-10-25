// Equalizer settings, defaults and presets. Kept apart from index.ts (which imports the
// Svelte components) so the form and tests can use them without an import cycle.

export type EqStyle = 'bars' | 'mirror' | 'glow' | 'led' | 'particles' | 'line' | 'radial';

export interface EqSettings {
  style: EqStyle;
  /** Size: length along the bar (px), share of the bar's thickness used (%), bars, gap and corner (px). */
  length: number; fill: number; bands: number; gap: number; radius: number;
  /** Style extras: LED segment height (px), particle-wave layers, line width (px), area fill under the line. */
  ledSize: number; layers: number; lineWidth: number; area: boolean;
  /** Colour. hueSpeed (°/s) slowly cycles rainbow colours. 'album' takes two colours from the song's cover. */
  colorMode: 'accent' | 'solid' | 'gradient' | 'rainbow' | 'album';
  color1: string; color2: string; direction: 'across' | 'up'; hueSpeed: number;
  glow: boolean; glowSize: number; glowStrength: number; glowColor: 'match' | 'custom'; glowCustom: string;
  /** Mirror image under the bars: strength (%) and share of the height it takes (%). */
  reflection: boolean; reflStrength: number; reflSize: number;
  /** Opacity, %. */
  barOpacity: number; bgOpacity: number; idleOpacity: number;
  /** Motion: gain %, fall speed 1–10, peak caps and how long they hang (ms), right-to-left. */
  gain: number; fall: number; peaks: boolean; peakHold: number; flip: boolean;
  /** Frequency range (Hz, strings for the choice buttons), bass/treble tilt −100..100, spacing. */
  lowHz: string; highHz: string; tilt: number; spacing: 'log' | 'linear';
  idle: 'dots' | 'line' | 'hide';
  fps: '30' | '60';
  /** Stop capturing and show the idle look while the laptop runs on battery. */
  batterySaver: boolean;
  myPresets: Preset[];
}

export interface Preset { name: string; values: Partial<EqSettings> }

export const EQ_DEFAULTS: EqSettings = {
  style: 'bars', length: 120, fill: 80, bands: 24, gap: 2, radius: 2,
  ledSize: 3, layers: 5, lineWidth: 2, area: true,
  colorMode: 'rainbow', color1: '#00e5ff', color2: '#ff2bd6', direction: 'across', hueSpeed: 0,
  glow: true, glowSize: 6, glowStrength: 60, glowColor: 'match', glowCustom: '#ffffff',
  reflection: false, reflStrength: 35, reflSize: 35,
  barOpacity: 100, bgOpacity: 0, idleOpacity: 50,
  gain: 100, fall: 4, peaks: true, peakHold: 600, flip: false,
  lowHz: '30', highHz: '16000', tilt: 30, spacing: 'log',
  idle: 'dots', fps: '60', batterySaver: true, myPresets: [],
};

/** Settings a preset carries: the look and motion, not the widget's size, frame rate or power options. */
const NOT_LOOK = new Set<keyof EqSettings>(['length', 'fps', 'batterySaver', 'myPresets', 'idle']);
export const pickLook = (s: EqSettings): Partial<EqSettings> =>
  Object.fromEntries(Object.entries(s).filter(([k]) => !NOT_LOOK.has(k as keyof EqSettings))) as Partial<EqSettings>;

/** A preset's patch: unset look settings go back to their defaults, so a preset always looks the same. */
export const applyPreset = (p: Preset): Partial<EqSettings> => ({ ...pickLook(EQ_DEFAULTS), ...p.values });

export const PRESETS: Preset[] = [
  { name: 'Neon', values: {
    style: 'glow', bands: 64, gap: 2, colorMode: 'gradient', color1: '#00e5ff', color2: '#ff2bd6',
    glowSize: 8, glowStrength: 80, peaks: false } },
  { name: 'Retro LED', values: {
    style: 'led', bands: 20, gap: 2, radius: 0, ledSize: 3, colorMode: 'gradient', direction: 'up',
    color1: '#22c55e', color2: '#ef4444', glow: false, peakHold: 800, reflection: true, reflStrength: 30, reflSize: 30 } },
  { name: 'Minimal', values: {
    style: 'bars', bands: 16, gap: 3, radius: 3, colorMode: 'accent', glow: false, peaks: false, barOpacity: 85 } },
  { name: 'Aurora', values: {
    style: 'line', bands: 48, lineWidth: 2, area: true, colorMode: 'rainbow', hueSpeed: 20,
    glowSize: 10, glowStrength: 70, peaks: false } },
  { name: 'Rainbow Wave', values: {
    style: 'particles', bands: 48, layers: 6, colorMode: 'rainbow', direction: 'up', glowSize: 4, glowStrength: 50 } },
];
