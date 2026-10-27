// What the companion feels, and how that draws.
//
// Pure on purpose: the priority order is the whole feature, and it has to be impossible for
// two triggers to both win. Everything here is a plain function of inputs the dock already
// knows -- no clocks, no subscriptions. The component owns the timers.

export type Mood =
  | 'neutral' | 'blink' | 'happy' | 'wink' | 'giggle' | 'angry' | 'sad'
  | 'sleepy' | 'asleep' | 'surprised' | 'love' | 'dizzy' | 'sing' | 'focus' | 'calm';

/** The things that can move it, each switchable on its own. */
export type Trigger = 'tickle' | 'charger' | 'battery' | 'music' | 'prayer' | 'pomodoro' | 'dnd' | 'idle';

export const TRIGGERS: { key: Trigger; label: string }[] = [
  { key: 'tickle', label: 'Tickling' },
  { key: 'charger', label: 'Charger' },
  { key: 'battery', label: 'Low battery' },
  { key: 'music', label: 'Music' },
  { key: 'prayer', label: 'Prayer time' },
  { key: 'pomodoro', label: 'Pomodoro' },
  { key: 'dnd', label: 'Do Not Disturb' },
  { key: 'idle', label: 'Falling asleep' },
];

export interface Inputs {
  /** A brief reaction the component is timing out; null the rest of the time. */
  reaction: 'click' | 'plugged' | 'unplugged' | null;
  /** How tickled it is right now, 0..1 (see tickle.ts). */
  tickle: number;
  plugged: boolean;
  percent: number | null;
  playing: boolean;
  /** A prayer time has just come. */
  prayerNow: boolean;
  pomo: 'work' | 'rest' | null;
  dnd: boolean;
  /** Nobody has touched the dock for a while. */
  asleep: boolean;
}

/** On battery at or below this, it looks worried. Matches the Battery widget's red band. */
export const LOW_PCT = 15;
/** Tickled past this and it gets dizzy rather than just giggly. */
export const DIZZY = 0.99;

/**
 * The one mood that wins.
 *
 * The order is the design: whatever the user just *did* beats whatever is merely true, and a
 * state the user chose (Do Not Disturb) beats the ambient ones. Charging last, because it
 * lasts for hours and would otherwise drown everything else out.
 */
export function moodFor(i: Inputs, on: Record<Trigger, boolean>): Mood {
  // 1. Being poked, right now.
  if (on.tickle && i.tickle > 0) return i.tickle >= DIZZY ? 'dizzy' : 'giggle';
  if (i.reaction === 'click') return 'wink';
  // 2. The charger just moved -- a moment's reaction, then it settles.
  if (on.charger && i.reaction === 'plugged') return 'love';
  if (on.charger && i.reaction === 'unplugged') return 'surprised';
  // 3. Things that matter more than mood.
  if (on.prayer && i.prayerNow) return 'calm';
  if (on.battery && !i.plugged && i.percent !== null && i.percent <= LOW_PCT) return 'sad';
  // 4. What the user is in the middle of.
  if (on.pomodoro && i.pomo === 'work') return 'focus';
  if (on.pomodoro && i.pomo === 'rest') return 'happy';
  if (on.music && i.playing) return 'sing';
  // 5. Quiet states.
  if (on.dnd && i.dnd) return 'sleepy';
  if (on.idle && i.asleep) return 'asleep';
  if (on.charger && i.plugged) return 'happy';
  return 'neutral';
}

/**
 * How a mood draws. Every field is a number or a small enum so the face can transition between
 * them instead of cutting -- the eyes are one pair of shapes that change, not 15 pictures.
 */
export interface Look {
  /** Eye height in viewBox units. The eye is a rounded rect always rounded to a capsule, so
   *  a tall one is an eye and a flat one is a shut one -- blinking is this one number moving.
   *  The radius follows from it (see EYE_W in Face.svelte) and is never a setting of its own,
   *  because a radius that doesn't match the height can only be wrong. */
  h: number;
  /** How far the lid comes down, 0..1. */
  lid: number;
  /** Lid tilt in degrees: positive glares, negative droops. */
  tilt: number;
  /** A shape instead of the round eye. */
  shape: 'none' | 'arc' | 'heart' | 'cross';
  mouth: 'none' | 'smile' | 'open' | 'flat' | 'frown';
  motion: 'none' | 'bob' | 'shiver' | 'sway';
}

const EYE: Look = { h: 11, lid: 0, tilt: 0, shape: 'none', mouth: 'none', motion: 'none' };

export const LOOKS: Record<Mood, Look> = {
  neutral: { ...EYE },
  blink: { ...EYE, h: 2 },
  happy: { ...EYE, shape: 'arc', mouth: 'smile' },
  wink: { ...EYE, mouth: 'smile' }, // the right eye is shut by the face, not here
  giggle: { ...EYE, shape: 'arc', mouth: 'smile', motion: 'shiver' },
  angry: { ...EYE, h: 9, lid: 0.35, tilt: 18, mouth: 'flat' },
  sad: { ...EYE, h: 10, lid: 0.2, tilt: -16, mouth: 'frown' },
  sleepy: { ...EYE, h: 11, lid: 0.55, mouth: 'flat' },
  asleep: { ...EYE, h: 2.5, mouth: 'none', motion: 'sway' },
  surprised: { ...EYE, h: 14, mouth: 'open' },
  love: { ...EYE, shape: 'heart', mouth: 'smile' },
  dizzy: { ...EYE, shape: 'cross', mouth: 'open', motion: 'sway' },
  sing: { ...EYE, shape: 'arc', mouth: 'open', motion: 'bob' },
  focus: { ...EYE, h: 6, lid: 0.15, mouth: 'flat' },
  calm: { ...EYE, h: 3, mouth: 'smile' },
};

/** Blinking is wrong for a mood whose eyes are already shut or aren't round. */
export const canBlink = (m: Mood): boolean =>
  LOOKS[m].shape === 'none' && LOOKS[m].h > 5 && m !== 'asleep' && m !== 'calm';
