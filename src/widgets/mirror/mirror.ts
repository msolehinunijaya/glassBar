// Mirror widget: the small pure bits (crop math, time text), kept here so they can be tested.

export type Shape = 'circle' | 'square' | 'rect';

export interface MirrorSettings {
  shape: Shape;
  /** Flip left↔right, like a real mirror. */
  flip: boolean;
  /** Percent, 50–200. */
  brightness: number;
  /** Percent, 50–200. */
  contrast: number;
  /** 1–3, a centre crop. */
  zoom: number;
  /** Ring light around the preview, 0 (off) – 100. */
  ring: number;
  grid: boolean;
  /** 3-second countdown before a photo. */
  countdown: boolean;
  /** Camera deviceId; '' = the default camera. */
  camera: string;
}

export const DEFAULTS: MirrorSettings = {
  shape: 'circle', flip: true, brightness: 100, contrast: 100, zoom: 1, ring: 0, grid: false, countdown: false, camera: '',
};

export interface Rect { x: number; y: number; w: number; h: number }

/** The part of a `vw`×`vh` frame that is shown: the centre, `zoom` times closer. A circle
 *  shows a square (the shorter side). The result is what a photo saves. */
export function cropOf(vw: number, vh: number, zoom: number, shape: Shape): Rect {
  const z = Math.min(3, Math.max(1, zoom || 1));
  let w = vw, h = vh;
  if (shape === 'circle' || shape === 'square') w = h = Math.min(vw, vh);
  w /= z; h /= z;
  return { x: (vw - w) / 2, y: (vh - h) / 2, w, h };
}

/** 75 → "1:15". */
export const clock = (sec: number) => `${Math.floor(sec / 60)}:${String(Math.floor(sec % 60)).padStart(2, '0')}`;

/** The CSS filter for the preview and the saved photo. */
export const filterOf = (s: Pick<MirrorSettings, 'brightness' | 'contrast'>) => `brightness(${s.brightness}%) contrast(${s.contrast}%)`;
