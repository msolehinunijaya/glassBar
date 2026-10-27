// The charger flash: plugging in or unplugging briefly turns the whole dock into a battery
// display. Pure, so the colours, the wording and the timing can be checked without a dock.
import type { ChargerFlip } from '../lib/ipc';
import { duration } from '../lib/time';

/** The Battery widget's palette, so the dock and the pill never disagree about a level. */
const GREEN = '#30d158', AMBER = '#ff9f0a', RED = '#ff453a';

/** On mains it is always green. On battery the colour *is* the level (DESIGN_RULES §1). */
export function flashTone(plugged: boolean, percent: number | null): string {
  if (plugged || percent === null) return GREEN;
  return percent <= 15 ? RED : percent <= 30 ? AMBER : GREEN;
}

/** How much of the dock the fill covers, 0..1. */
export const fillOf = (percent: number | null): number => (percent === null ? 1 : Math.max(0, Math.min(100, percent)) / 100);

/**
 * The four phases, in ms. `enter` is the 150-200 ms scale-and-fade DESIGN_RULES §4 asks for;
 * `fill` sweeps the level in; `hold` is the part the user actually reads; `exit` fades out.
 * The timeouts own this schedule, never a transitionend — with motion turned off there are
 * no transitions to wait for.
 */
export const FLASH = { enter: 180, fill: 700, hold: 1400, exit: 220 } as const;
export const FLASH_MS = FLASH.enter + FLASH.fill + FLASH.hold + FLASH.exit;

/** The big number and the line under it. Wording matches the Battery panel. */
export function flashText(f: ChargerFlip): { pct: string; label: string } {
  const pct = f.percent === null ? '--' : `${f.percent}%`;
  const state = f.plugged ? (f.percent !== null && f.percent >= 95 ? 'Plugged in' : 'Charging') : 'On battery';
  const left = !f.plugged && f.secsLeft ? ` · about ${duration(f.secsLeft)} left` : '';
  return { pct, label: state + left };
}
