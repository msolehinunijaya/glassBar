import { describe, expect, it, vi } from 'vitest';
import { playAdzan, stopSound } from './sound';

class FakeAudio extends EventTarget {
  static last: FakeAudio;
  currentTime = 0; volume = 1;
  constructor() { super(); FakeAudio.last = this; }
  play = () => Promise.resolve();
  pause = () => {};
}
vi.stubGlobal('Audio', FakeAudio);
const flush = () => new Promise((r) => setTimeout(r));

describe('playAdzan', () => {
  it('pauses playing media, then resumes those apps when the adzan ends', async () => {
    const media = { pause: vi.fn(async () => ['Spotify.exe']), resume: vi.fn(async () => {}) };
    playAdzan('a.mp3', false, true, media);
    expect(media.pause).toHaveBeenCalledOnce();
    await flush();
    expect(media.resume).not.toHaveBeenCalled();
    FakeAudio.last.dispatchEvent(new Event('ended'));
    await flush();
    expect(media.resume).toHaveBeenCalledWith(['Spotify.exe']);
  });

  it('resumes when the adzan is stopped, and only once', async () => {
    const media = { pause: vi.fn(async () => ['x']), resume: vi.fn(async () => {}) };
    playAdzan('a.mp3', true, true, media);
    await flush();
    stopSound();
    stopSound();
    FakeAudio.last.dispatchEvent(new Event('ended'));
    await flush();
    expect(media.resume).toHaveBeenCalledOnce();
  });

  it('starts the adzan only after the pause is done', async () => {
    let done!: (ids: string[]) => void;
    const media = { pause: vi.fn(() => new Promise<string[]>((r) => (done = r))), resume: vi.fn(async () => {}) };
    const before = FakeAudio.last;
    playAdzan('a.mp3', false, true, media);
    expect(FakeAudio.last).toBe(before);
    done([]);
    await flush();
    expect(FakeAudio.last).not.toBe(before);
    stopSound();
  });

  it('leaves media alone when off, or when nothing was playing', async () => {
    const off = { pause: vi.fn(async () => ['x']), resume: vi.fn(async () => {}) };
    playAdzan('a.mp3', false, false, off);
    stopSound();
    const idle = { pause: vi.fn(async () => []), resume: vi.fn(async () => {}) };
    playAdzan('a.mp3', false, true, idle);
    await flush();
    stopSound();
    await flush();
    expect(off.pause).not.toHaveBeenCalled();
    expect(off.resume).not.toHaveBeenCalled();
    expect(idle.resume).not.toHaveBeenCalled();
  });
});
