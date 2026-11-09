import { describe, expect, it } from 'vitest';
import { nextIndex, playerUrl, youtubeId } from './music';

describe('youtubeId', () => {
  it('reads every common YouTube link', () => {
    for (const l of [
      'https://www.youtube.com/watch?v=jfKfPfyJRdk', 'https://youtube.com/watch?v=jfKfPfyJRdk&list=PL1&t=30',
      'https://youtu.be/jfKfPfyJRdk?si=abc', 'https://m.youtube.com/watch?v=jfKfPfyJRdk',
      'https://www.youtube.com/live/jfKfPfyJRdk', 'https://youtube.com/shorts/jfKfPfyJRdk', ' https://www.youtube.com/embed/jfKfPfyJRdk ',
    ]) expect(youtubeId(l), l).toBe('jfKfPfyJRdk');
  });
  it('rejects anything else', () => {
    for (const l of ['', 'jfKfPfyJRdk', 'https://vimeo.com/123', 'https://www.youtube.com/@lofigirl', 'https://evil.com/watch?v=jfKfPfyJRdk'])
      expect(youtubeId(l), l).toBeNull();
  });
});

describe('playerUrl', () => {
  it('starts on the first link, ready to be driven', () => {
    const u = new URL(playerUrl('aaaaaaaaaaa', 'http://tauri.localhost'));
    expect(u.pathname).toBe('/embed/aaaaaaaaaaa');
    expect(u.searchParams.get('autoplay')).toBe('1');
    expect(u.searchParams.get('enablejsapi')).toBe('1');
  });
  it('goes through the links in order, then starts over', () => {
    expect([0, 1, 2].map((i) => nextIndex(i, 3))).toEqual([1, 2, 0]);
    expect(nextIndex(0, 1)).toBe(0);
  });
});
