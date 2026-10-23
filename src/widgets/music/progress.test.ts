import { describe, expect, it } from 'vitest';
import { songProgress } from './progress';

const m = (p: Partial<{ playing: boolean; positionMs: number; durationMs: number; updatedAtMs: number }>) =>
  ({ app: '', title: 't', artist: '', album: '', art: null, canNext: true, canPrev: true, playing: true, positionMs: 0, durationMs: 200_000, updatedAtMs: 0, ...p });

describe('songProgress', () => {
  it('is position / duration, extrapolated while playing', () => {
    expect(songProgress(m({ positionMs: 50_000, updatedAtMs: 1000 }), 51_000)).toEqual({ frac: 0.5, ending: false });
  });
  it('flags the last 10 seconds', () => {
    expect(songProgress(m({ positionMs: 195_000, updatedAtMs: 0 }), 0)?.ending).toBe(true);
  });
  it('hides when the duration is unknown (live stream)', () => {
    expect(songProgress(m({ durationMs: 0 }), 0)).toBeNull();
  });
});
