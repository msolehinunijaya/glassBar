import { describe, expect, it } from 'vitest';
import { CUSTOM_DEFAULTS, MUSIC_DEFAULTS, artRadius, layoutOf, type MusicSettings } from './settings';

const S = (o: Partial<MusicSettings> = {}): MusicSettings => ({ ...MUSIC_DEFAULTS, ...o });

describe('the Player design is unchanged', () => {
  // These are the rules PlayerPill has always used: wide = not small and not a side dock.
  it('shows everything at medium and large', () => {
    for (const size of ['medium', 'large'] as const) {
      const p = layoutOf(S(), size, false);
      expect(p).toMatchObject({ art: true, title: true, artist: true, elapsed: true, duration: true, progress: true, prev: true, playPause: true, next: true, oneLine: true });
    }
  });

  it('drops the text, the progress line and Previous at small', () => {
    const p = layoutOf(S(), 'small', false);
    expect(p).toMatchObject({ art: true, title: false, artist: false, elapsed: false, duration: false, progress: false, prev: false, viz: 'none' });
    expect(p).toMatchObject({ playPause: true, next: true }); // Play/Pause and Next survive every size
  });

  it('shows the volume button only at large on a top or bottom dock', () => {
    expect(layoutOf(S(), 'large', false).volume).toBe(true);
    expect(layoutOf(S(), 'medium', false).volume).toBe(false);
    expect(layoutOf(S(), 'large', true).volume).toBe(false);
  });

  it('keeps only the art and two buttons on a side dock', () => {
    const p = layoutOf(S(), 'large', true);
    expect(p).toMatchObject({ art: true, title: false, progress: false, prev: false, viz: 'none', volume: false, playPause: true, next: true });
  });

  it('follows the level-visualizer switch and its width', () => {
    expect(layoutOf(S({ showViz: true }), 'medium', false).viz).toBe('level');
    expect(layoutOf(S({ showViz: false }), 'medium', false).viz).toBe('none');
    expect(layoutOf(S({ showViz: true }), 'small', false).viz).toBe('none');
    expect(layoutOf(S({ vizWidth: 120 }), 'medium', false).vizWidth).toBe(120);
  });

  it('never reads the custom switches', () => {
    const p = layoutOf(S({ custom: { ...CUSTOM_DEFAULTS, art: false, next: false } }), 'medium', false);
    expect(p).toMatchObject({ art: true, next: true });
  });
});

describe('the custom design', () => {
  it('uses the switches the user set', () => {
    const p = layoutOf(S({ style: 'custom', custom: { ...CUSTOM_DEFAULTS, art: false, album: true, remaining: true, prev: false } }), 'medium', false);
    expect(p).toMatchObject({ art: false, album: true, remaining: true, prev: false, oneLine: false });
  });

  it('ignores the widget size, because the user chose the parts', () => {
    const a = layoutOf(S({ style: 'custom' }), 'small', false);
    const b = layoutOf(S({ style: 'custom' }), 'large', false);
    expect(a).toEqual(b);
  });

  it('fills in parts a config saved earlier never knew about', () => {
    // settingsOf() merges only one level deep, so `custom` arrives whole or not at all.
    const partial = { art: false } as unknown as MusicSettings['custom'];
    const p = layoutOf(S({ style: 'custom', custom: partial }), 'medium', false);
    expect(p.art).toBe(false); // what was saved wins
    expect(p.playPause).toBe(CUSTOM_DEFAULTS.playPause);
    expect(p.title).toBe(CUSTOM_DEFAULTS.title);
    expect(p.artShape).toBe(CUSTOM_DEFAULTS.artShape);
    expect(p.viz).toBe(CUSTOM_DEFAULTS.viz);
  });

  it('survives a config with no custom block at all', () => {
    const p = layoutOf({ ...MUSIC_DEFAULTS, style: 'custom', custom: undefined as unknown as MusicSettings['custom'] }, 'medium', false);
    expect(p).toMatchObject({ ...CUSTOM_DEFAULTS, oneLine: false });
  });

  it('drops what cannot fit a side dock, whatever the user chose', () => {
    const all = { ...CUSTOM_DEFAULTS, title: true, artist: true, album: true, appName: true, remaining: true, viz: 'spectrum' as const, progress: true };
    const p = layoutOf(S({ style: 'custom', custom: all }), 'medium', true);
    expect(p).toMatchObject({ title: false, artist: false, album: false, appName: false, elapsed: false, remaining: false, duration: false, viz: 'none', progress: false });
    expect(p).toMatchObject({ art: true, playPause: true }); // the art and the buttons still fit
  });

  it('starts out looking like the Player design', () => {
    const custom = layoutOf(S({ style: 'custom' }), 'medium', false);
    const player = layoutOf(S(), 'medium', false);
    for (const k of ['art', 'title', 'artist', 'elapsed', 'duration', 'progress', 'prev', 'playPause', 'next', 'viz'] as const) {
      expect(custom[k]).toBe(player[k]);
    }
  });
});

describe('artRadius', () => {
  it('rounds the art to each shape, following the chosen corner radius', () => {
    expect(artRadius('circle')).toBe('50%');
    expect(artRadius('rounded')).toContain('var(--rs)');
    expect(artRadius('square')).toContain('var(--rs)');
    expect(artRadius('square')).not.toBe(artRadius('rounded'));
  });
});
