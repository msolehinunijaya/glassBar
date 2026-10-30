// Shared by the pill and the panel: the clock, and repeating countdowns rolled forward.
import { updateWidgetSettings } from '../../lib/config.svelte';
import { left, occurrence, progressOf, rollOver, type Countdown } from './dates';
import type { CountdownSettings } from './index';

/** A clock that ticks each minute (each 15 s when something is within the hour). */
export function countdownClock(items: () => Countdown[]) {
  const s = $state({ now: new Date() });
  $effect(() => {
    const soon = items().some((c) => { const ms = occurrence(c, s.now).getTime() - s.now.getTime(); return c.time && ms > 0 && ms < 3_600_000; });
    const t = setTimeout(() => (s.now = new Date()), soon ? 15_000 : 60_000 - (Date.now() % 60_000));
    return () => clearTimeout(t);
  });
  return s;
}

/** Move repeating countdowns whose day is over to their next date (saved once). */
export function keepRolled(instance: () => string, settings: () => CountdownSettings, now: () => Date) {
  $effect(() => {
    const items = settings().items ?? [];
    const rolled = items.map((c) => rollOver(c, now()));
    if (rolled.some((c, i) => c !== items[i])) updateWidgetSettings(instance(), { items: rolled });
  });
}

export interface View { c: Countdown; target: Date; l: ReturnType<typeof left>; progress: number }

/** Every countdown with its time left, soonest first, finished ones last. */
export function views(items: Countdown[], now: Date): View[] {
  return items.map((c) => { const target = occurrence(c, now); return { c, target, l: left(target, !!c.time, now), progress: progressOf(c.created, target, now) }; })
    .sort((a, b) => Number(a.l.state === 'done') - Number(b.l.state === 'done') || a.target.getTime() - b.target.getTime());
}
