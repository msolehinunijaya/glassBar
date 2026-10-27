// Music settings, defaults and the layout resolver. Kept apart from index.ts (which imports
// the Svelte components) so the form and the tests can use them without an import cycle.
import type { WidgetSize } from '../../lib/ipc';
import type { EqStyle } from '../equalizer/settings';

export type MusicStyle = 'player' | 'classic' | 'mini' | 'custom';
/** Album art outline. */
export type ArtShape = 'circle' | 'rounded' | 'square';
/** none · the output level meter (cheap) · the real frequency spectrum (the Equalizer widget's). */
export type MusicViz = 'none' | 'level' | 'spectrum';

/** What the user turns on and off in the custom design. */
export interface MusicParts {
  art: boolean; artShape: ArtShape; artSize: number;
  title: boolean; artist: boolean; album: boolean; appName: boolean;
  elapsed: boolean; remaining: boolean; duration: boolean;
  viz: MusicViz; vizWidth: number; eqLook: EqStyle;
  progress: boolean;
  prev: boolean; playPause: boolean; next: boolean; volume: boolean;
}

/** What PlayerPill draws: the parts, plus how the text is composed. The Player design puts
 *  the artist and the times on one line; the custom design gives each part its own line. */
export type MusicLayout = MusicParts & { oneLine: boolean };

export interface MusicSettings {
  style: MusicStyle;
  /** Classic and Mini only (the custom design has its own switches). */
  showControls: boolean; showProgress: boolean;
  /** Player only: the level visualizer and its width in px. */
  showViz: boolean; vizWidth: number;
  custom: MusicParts;
}

/** Custom starts out looking like the Player design, so turning it on never shows a blank pill. */
export const CUSTOM_DEFAULTS: MusicParts = {
  art: true, artShape: 'rounded', artSize: 28,
  title: true, artist: true, album: false, appName: false,
  elapsed: true, remaining: false, duration: true,
  viz: 'level', vizWidth: 64, eqLook: 'bars',
  progress: true,
  prev: true, playPause: true, next: true, volume: false,
};

export const MUSIC_DEFAULTS: MusicSettings = {
  style: 'player', showControls: true, showProgress: true, showViz: true, vizWidth: 64,
  custom: CUSTOM_DEFAULTS,
};

/** Nothing that needs horizontal room survives a 56-96 px side dock (DESIGN_RULES §8). */
const NARROW = { title: false, artist: false, album: false, appName: false,
  elapsed: false, remaining: false, duration: false, viz: 'none', progress: false } as const;

/**
 * Which parts PlayerPill draws. For `custom` these are the user's switches; for `player` they
 * are the size-driven rules the design has always used, so the Player design is unchanged.
 *
 * `settingsOf()` merges a widget's saved settings only one level deep, so `custom` arrives
 * whole or not at all -- it is merged over the defaults here by hand. Without this, a config
 * saved before a part existed would read that part as `undefined`, i.e. off.
 */
export function layoutOf(s: MusicSettings, size: WidgetSize, vertical: boolean): MusicLayout {
  if (s.style === 'custom') {
    const c: MusicLayout = { ...CUSTOM_DEFAULTS, ...(s.custom ?? {}), oneLine: false };
    return vertical ? { ...c, ...NARROW } : c;
  }
  const wide = size !== 'small' && !vertical;
  return {
    art: true, artShape: 'rounded', artSize: 28,
    title: wide, artist: wide, album: false, appName: false,
    elapsed: wide, remaining: false, duration: wide,
    viz: wide && s.showViz ? 'level' : 'none', vizWidth: s.vizWidth, eqLook: 'bars',
    progress: wide,
    prev: wide, playPause: true, next: true,
    volume: size === 'large' && !vertical,
    oneLine: true,
  };
}

/** The art's corner radius, as a CSS length. */
export const artRadius = (shape: ArtShape): string =>
  shape === 'circle' ? '50%' : shape === 'square' ? 'calc(2px * var(--rs))' : 'calc(8px * var(--rs))';
