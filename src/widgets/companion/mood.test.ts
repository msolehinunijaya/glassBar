import { describe, expect, it } from 'vitest';
import { DIZZY, LOOKS, LOW_PCT, TRIGGERS, canBlink, moodFor, type Inputs, type Mood, type Trigger } from './mood';

const ALL: Record<Trigger, boolean> = { tickle: true, charger: true, battery: true, music: true, prayer: true, pomodoro: true, dnd: true, idle: true };
const NONE: Record<Trigger, boolean> = { tickle: false, charger: false, battery: false, music: false, prayer: false, pomodoro: false, dnd: false, idle: false };

const calm: Inputs = {
  reaction: null, tickle: 0, plugged: false, percent: 80, playing: false,
  prayerNow: false, pomo: null, dnd: false, asleep: false,
};
const I = (o: Partial<Inputs> = {}): Inputs => ({ ...calm, ...o });

describe('what it feels', () => {
  it('just sits there when nothing is happening', () => {
    expect(moodFor(I(), ALL)).toBe('neutral');
  });

  it('giggles when tickled, and goes dizzy if you keep going', () => {
    expect(moodFor(I({ tickle: 0.4 }), ALL)).toBe('giggle');
    expect(moodFor(I({ tickle: DIZZY }), ALL)).toBe('dizzy');
    expect(moodFor(I({ tickle: 1 }), ALL)).toBe('dizzy');
  });

  it('winks when clicked', () => {
    expect(moodFor(I({ reaction: 'click' }), ALL)).toBe('wink');
  });

  it('reacts to the charger, then settles into happy', () => {
    expect(moodFor(I({ reaction: 'plugged', plugged: true }), ALL)).toBe('love');
    expect(moodFor(I({ reaction: 'unplugged' }), ALL)).toBe('surprised');
    expect(moodFor(I({ plugged: true }), ALL)).toBe('happy');
  });

  it('goes calm at prayer time', () => {
    expect(moodFor(I({ prayerNow: true }), ALL)).toBe('calm');
  });

  it('looks worried on a low battery, but not while charging', () => {
    expect(moodFor(I({ percent: LOW_PCT }), ALL)).toBe('sad');
    expect(moodFor(I({ percent: LOW_PCT + 1 }), ALL)).toBe('neutral');
    expect(moodFor(I({ percent: 5, plugged: true }), ALL)).toBe('happy');
  });

  it('never invents a mood for a PC with no battery', () => {
    expect(moodFor(I({ percent: null }), ALL)).toBe('neutral');
  });

  it('concentrates during a Pomodoro and cheers up on the break', () => {
    expect(moodFor(I({ pomo: 'work' }), ALL)).toBe('focus');
    expect(moodFor(I({ pomo: 'rest' }), ALL)).toBe('happy');
  });

  it('sings along to music', () => {
    expect(moodFor(I({ playing: true }), ALL)).toBe('sing');
  });

  it('gets sleepy on Do Not Disturb and nods off when left alone', () => {
    expect(moodFor(I({ dnd: true }), ALL)).toBe('sleepy');
    expect(moodFor(I({ asleep: true }), ALL)).toBe('asleep');
  });
});

describe('the priority order', () => {
  it('lets a poke beat everything that is merely true', () => {
    const busy = I({ tickle: 0.5, plugged: true, playing: true, prayerNow: true, pomo: 'work', dnd: true, asleep: true, percent: 5 });
    expect(moodFor(busy, ALL)).toBe('giggle');
  });

  it('puts prayer time above a low battery and a Pomodoro', () => {
    expect(moodFor(I({ prayerNow: true, percent: 3, pomo: 'work' }), ALL)).toBe('calm');
  });

  it('puts a low battery above music', () => {
    expect(moodFor(I({ percent: 4, playing: true }), ALL)).toBe('sad');
  });

  it('does not let hours of charging drown out the rest', () => {
    expect(moodFor(I({ plugged: true, playing: true }), ALL)).toBe('sing');
    expect(moodFor(I({ plugged: true, pomo: 'work' }), ALL)).toBe('focus');
    expect(moodFor(I({ plugged: true, dnd: true }), ALL)).toBe('sleepy');
  });

  it('wakes up for anything, rather than staying asleep', () => {
    expect(moodFor(I({ asleep: true, playing: true }), ALL)).toBe('sing');
    expect(moodFor(I({ asleep: true, reaction: 'click' }), ALL)).toBe('wink');
  });

  it('resolves to exactly one mood for every combination it can be handed', () => {
    // 2^6 x 4 states: the point is that nothing throws and nothing comes back undefined.
    const flags = [false, true];
    for (const plugged of flags) for (const playing of flags) for (const prayerNow of flags)
      for (const dnd of flags) for (const asleep of flags) for (const low of flags)
        for (const pomo of [null, 'work', 'rest'] as const)
          for (const reaction of [null, 'click', 'plugged', 'unplugged'] as const) {
            const m = moodFor(I({ plugged, playing, prayerNow, dnd, asleep, pomo, reaction, percent: low ? 5 : 80 }), ALL);
            expect(LOOKS[m]).toBeDefined();
          }
  });
});

describe('the switches', () => {
  it('stays neutral with every trigger turned off, whatever is going on', () => {
    const busy = I({ tickle: 1, reaction: 'plugged', plugged: true, playing: true, prayerNow: true, pomo: 'work', dnd: true, asleep: true, percent: 2 });
    // A click is the companion's own button, not a trigger you can switch off.
    expect(moodFor({ ...busy, reaction: null }, NONE)).toBe('neutral');
  });

  it('turns off one trigger without touching the others', () => {
    const noMusic = { ...ALL, music: false };
    expect(moodFor(I({ playing: true }), noMusic)).toBe('neutral');
    expect(moodFor(I({ playing: true, pomo: 'work' }), noMusic)).toBe('focus');
  });

  it('offers a switch for every trigger the resolver reads', () => {
    expect(TRIGGERS.map((t) => t.key).sort()).toEqual(Object.keys(ALL).sort());
  });
});

describe('how a mood draws', () => {
  it('describes every mood', () => {
    const moods: Mood[] = ['neutral', 'blink', 'happy', 'wink', 'giggle', 'angry', 'sad', 'sleepy', 'asleep', 'surprised', 'love', 'dizzy', 'sing', 'focus', 'calm'];
    for (const m of moods) expect(LOOKS[m], m).toBeDefined();
    expect(Object.keys(LOOKS).sort()).toEqual([...moods].sort());
  });

  it('keeps every eye and lid within sane bounds', () => {
    for (const [m, l] of Object.entries(LOOKS)) {
      expect(l.h, m).toBeGreaterThan(0);
      expect(l.h, m).toBeLessThanOrEqual(16);
      expect(l.lid, m).toBeGreaterThanOrEqual(0);
      expect(l.lid, m).toBeLessThanOrEqual(1);
      expect(Math.abs(l.tilt), m).toBeLessThanOrEqual(30);
    }
  });

  it('only blinks when there are open round eyes to blink with', () => {
    expect(canBlink('neutral')).toBe(true);
    expect(canBlink('surprised')).toBe(true);
    expect(canBlink('sleepy')).toBe(true);
    expect(canBlink('asleep')).toBe(false);
    expect(canBlink('calm')).toBe(false);
    expect(canBlink('happy')).toBe(false); // arc eyes
    expect(canBlink('love')).toBe(false);
    expect(canBlink('dizzy')).toBe(false);
    expect(canBlink('blink')).toBe(false); // already shut
  });
});
