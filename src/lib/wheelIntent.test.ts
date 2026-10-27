import { describe, expect, it } from 'vitest';
import { wheelIntent } from './wheelIntent';

// A "node" contains the targets listed in it.
const node = (...kids: string[]) => ({ contains: (n: unknown) => kids.includes(n as string) });
const vol = node('speaker');

describe('wheelIntent', () => {
  it('a scroll that started on the dock never changes the volume on the way past', () => {
    const w = wheelIntent();
    w.seen('dock', 1000, 0, 100);
    w.seen('speaker', 1100, 0, 100); // pointer drifted over the speaker mid-gesture
    expect(w.owns(vol, 0)).toBe(false);
  });

  it('a scroll that starts on the speaker after resting there changes the volume', () => {
    const w = wheelIntent();
    w.seen('speaker', 1000, 0, -100);
    expect(w.owns(vol, 700)).toBe(true);
  });

  it('arriving and scrolling at once is still a dock scroll', () => {
    const w = wheelIntent();
    w.seen('speaker', 1000, 0, -100);
    expect(w.owns(vol, 900)).toBe(false);
  });

  it('a pause longer than the gap starts a new gesture', () => {
    const w = wheelIntent();
    w.seen('dock', 1000, 0, 100);
    w.seen('speaker', 1600, 0, 100);
    expect(w.owns(vol, 1200)).toBe(true);
  });

  it('a sideways swipe never changes the volume', () => {
    const w = wheelIntent();
    w.seen('speaker', 1000, 80, 10);
    expect(w.owns(vol, 0)).toBe(false);
  });
});
