import { describe, expect, it } from 'vitest';
import type { MediaInfo } from '../../lib/ipc';
import { subText, timesText } from './label';
import { MUSIC_DEFAULTS, layoutOf, type MusicSettings } from './settings';

const song = (o: Partial<MediaInfo> = {}): MediaInfo => ({
  app: 'Spotify', title: 'Sprinter', artist: 'Dave', album: 'Split Decision', art: null,
  playing: false, positionMs: 83_000, durationMs: 296_000, updatedAtMs: 0, canNext: true, canPrev: true, ...o,
});
const player = (size: 'small' | 'medium' | 'large' = 'medium') => layoutOf(MUSIC_DEFAULTS, size, false);
const custom = (parts: Partial<MusicSettings['custom']> = {}) =>
  layoutOf({ ...MUSIC_DEFAULTS, style: 'custom', custom: { ...MUSIC_DEFAULTS.custom, ...parts } }, 'medium', false);

describe('timesText', () => {
  it('reads elapsed / total, the way the Player design always has', () => {
    expect(timesText(player(), song(), 0)).toBe('1:23 / 4:56');
  });

  it('can count down instead', () => {
    expect(timesText(custom({ elapsed: true, remaining: true, duration: false }), song(), 0)).toBe('1:23 / -3:33');
  });

  it('shows one time on its own', () => {
    expect(timesText(custom({ elapsed: false, remaining: false, duration: true }), song(), 0)).toBe('4:56');
    expect(timesText(custom({ elapsed: true, remaining: false, duration: false }), song(), 0)).toBe('1:23');
  });

  it('shows nothing when every time is off', () => {
    expect(timesText(custom({ elapsed: false, remaining: false, duration: false }), song(), 0)).toBe('');
  });

  it('invents no time for a live stream', () => {
    expect(timesText(player(), song({ durationMs: 0 }), 0)).toBe('');
  });

  it('is empty with nothing playing', () => {
    expect(timesText(player(), null, 0)).toBe('');
  });
});

describe('subText', () => {
  it('reads artist and the times on the Player design', () => {
    expect(subText(player(), song(), 0)).toBe('Dave · 1:23 / 4:56');
  });

  it('falls back to the app name when the player gives no artist', () => {
    expect(subText(player(), song({ artist: '' }), 0)).toBe('Spotify · 1:23 / 4:56');
  });

  it('keeps artist and app apart on the custom design', () => {
    expect(subText(custom({ artist: true, appName: true }), song({ artist: '' }), 0)).toBe('Spotify · 1:23 / 4:56');
    expect(subText(custom({ artist: true, appName: true }), song(), 0)).toBe('Dave · Spotify · 1:23 / 4:56');
  });

  it('adds the album when asked', () => {
    expect(subText(custom({ album: true }), song(), 0)).toBe('Dave · Split Decision · 1:23 / 4:56');
  });

  it('drops facts the player left blank rather than showing gaps', () => {
    expect(subText(custom({ album: true, appName: true }), song({ album: '', app: '' }), 0)).toBe('Dave · 1:23 / 4:56');
  });

  it('is empty when every fact is off', () => {
    expect(subText(custom({ artist: false, album: false, appName: false, elapsed: false, remaining: false, duration: false }), song(), 0)).toBe('');
  });

  it('offers "Play something" under a title, and nothing without one', () => {
    // The regression this guards: a small Player pill has no text, so it must stay silent.
    expect(subText(player('medium'), null, 0)).toBe('Play something');
    expect(subText(player('small'), null, 0)).toBe('');
    expect(subText(custom({ title: false }), null, 0)).toBe('');
  });

  it('moves with the song while it plays', () => {
    const playing = song({ playing: true, positionMs: 83_000, updatedAtMs: 1_000 });
    expect(subText(player(), playing, 1_000)).toBe('Dave · 1:23 / 4:56');
    expect(subText(player(), playing, 11_000)).toBe('Dave · 1:33 / 4:56');
  });
});
