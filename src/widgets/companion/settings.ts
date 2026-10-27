// Companion settings and defaults. Kept apart from index.ts (which imports the Svelte
// components) so the form and the tests can use them without an import cycle.
import { TRIGGERS, type Trigger } from './mood';

export type Backdrop = 'none' | 'glass' | 'solid' | 'accent';

export interface CompanionSettings {
  /** Face height in px. The pill is as tall as the dock, so this is the face inside it. */
  size: number;
  backdrop: Backdrop;
  /** Used when the backdrop is 'solid'. */
  color: string;
  eyeColor: string;
  hands: boolean;
  /** Seconds between blinks, give or take: it is jittered so it never looks metronomic. */
  blinkEvery: number;
  /** Each trigger on its own, so it can react to only what you want. */
  on: Record<Trigger, boolean>;
}

export const COMPANION_DEFAULTS: CompanionSettings = {
  size: 24,
  backdrop: 'glass',
  color: '#1f6feb',
  eyeColor: '#ffffff',
  hands: true,
  blinkEvery: 5,
  on: Object.fromEntries(TRIGGERS.map((t) => [t.key, true])) as Record<Trigger, boolean>,
};

/** `settingsOf()` merges one level deep, so the trigger block is merged over its defaults
 *  here by hand -- otherwise a trigger added later would read as off, not as its default. */
export const triggersOf = (s: CompanionSettings): Record<Trigger, boolean> =>
  ({ ...COMPANION_DEFAULTS.on, ...(s.on ?? {}) });

/** How long a one-off reaction (a click, the charger moving) stays on the face. */
export const REACT_MS = 1600;
/** Nobody has touched the dock for this long: it nods off. */
export const IDLE_MS = 90_000;
