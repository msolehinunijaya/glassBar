import { describe, expect, it } from 'vitest';
import type { PomoState } from '../../lib/ipc';
import { contrast, DARK_INK, LIGHT_INK, PHASE_HEX, phaseTone, shadeFor } from './colors';
import { view } from './index';

const state = (pomo: Partial<PomoState['pomo']> = {}, today = { sessions: 0, minutes: 0 }): PomoState => ({
  pomo: { phase: 'work', done: 0, endsAt: null, pausedMs: null, completed: null, ...pomo },
  cfg: { workMin: 25, shortMin: 5, longMin: 15, every: 4, autoStart: false, sound: true, volume: 60, goal: 4, dimOthers: false },
  label: '', today: { day: '2026-10-24', longest: 25, ...today }, week: [], streak: 0, longest: 25,
});

describe('view', () => {
  it('names the state and the primary button', () => {
    expect(view(state())).toMatchObject({ status: 'ready', label: 'Ready', primary: 'Start focus', skip: 'Skip to break', idle: true });
    expect(view(state({ endsAt: 1 }))).toMatchObject({ label: 'Focusing', primary: 'Pause', idle: false });
    expect(view(state({ pausedMs: 5 }))).toMatchObject({ label: 'Paused', primary: 'Resume' });
    expect(view(state({ phase: 'short', done: 1, completed: 'work' }))).toMatchObject({ label: 'Ready', primary: 'Start break', skip: 'Skip to focus', idle: false });
    expect(view(state({ phase: 'long', endsAt: 1, done: 4 }))).toMatchObject({ label: 'Long break' });
    expect(view(state({ phase: 'short', endsAt: 1 })).label).toBe('Short break');
  });

  it('dots, session text and count agree', () => {
    const v = view(state({ done: 2, endsAt: 1 }, { sessions: 2, minutes: 50 }));
    expect(v.dots).toEqual(['done', 'done', 'current', 'upcoming']);
    expect(v.session).toBe('Session 3 of 4 · then short break');
    expect(v.count).toBe('3/4');
    expect(v.today).toMatchObject({ sessions: 2, minutes: 50, goal: 4, pct: 0.5, text: '50 min · 2 sessions' });
    expect(view(state({ done: 3 })).session).toBe('Session 4 of 4 · then long break');
    const brk = view(state({ phase: 'short', done: 2 }));
    expect([brk.dots, brk.session, brk.count]).toEqual([['done', 'done', 'upcoming', 'upcoming'], '2 of 4 done · next: session 3', '2/4']);
    const long = view(state({ phase: 'long', done: 4 }));
    expect([long.dots.every((d) => d === 'done'), long.session]).toEqual([true, '4 of 4 done · then a new round']);
    expect(view(state({}, { sessions: 9, minutes: 225 })).today).toMatchObject({ pct: 1, text: '3 h 45 min · 9 sessions' });
    expect(view(state({}, { sessions: 1, minutes: 25 })).today.text).toBe('25 min · 1 session');
  });

  it('is not idle once anything happened today', () => {
    expect(view(state({}, { sessions: 1, minutes: 25 })).idle).toBe(false);
    expect(view(state({ done: 1 })).idle).toBe(false);
  });
});

describe('phase colours', () => {
  it('text on the phase colour reaches 4.5:1 in both themes', () => {
    for (const p of ['work', 'short', 'long'] as const) for (const t of ['light', 'dark'] as const) {
      const { bg, ink } = phaseTone(p, t);
      expect(contrast(bg, ink), `${p} ${t}`).toBeGreaterThanOrEqual(4.5);
    }
    expect(phaseTone('work', 'dark')).toEqual({ bg: PHASE_HEX.work, ink: DARK_INK });
  });

  it('darkens only as far as needed', () => {
    const s = shadeFor('#FF6B57', LIGHT_INK);
    expect(s).not.toBe('#FF6B57');
    expect(contrast(s, LIGHT_INK)).toBeLessThan(4.7);
    expect(shadeFor('#000000', LIGHT_INK)).toBe('#000000');
    expect(contrast('#ffffff', '#000000')).toBeCloseTo(21);
  });

  it('glass.css carries the computed tokens', async () => {
    // (vitest blanks CSS imports, so read the file; node types aren't installed, hence untyped)
    const fs = 'node:fs';
    const css: string = (await import(/* @vite-ignore */ fs)).readFileSync('src/styles/glass.css', 'utf8');
    const block = (t: string) => css.slice(css.indexOf(`:root[data-theme='${t}'] {`)).split('}')[0];
    const names = { work: 'focus', short: 'short', long: 'long' } as const;
    for (const t of ['light', 'dark'] as const) {
      for (const p of ['work', 'short', 'long'] as const) expect(block(t)).toContain(`--phase-${names[p]}: ${phaseTone(p, t).bg};`);
      expect(block(t)).toContain(`--phase-ink: ${phaseTone('work', t).ink};`);
    }
  });
});
