// Limits each performance mode puts on top of a widget's own settings (Settings › Performance).
import type { PerfMode } from './ipc';

/** Frame-rate ceiling: Low 15, Normal 30 (or the setting if lower), High the user's setting. */
export function fpsCap(mode: PerfMode, userFps: number): number {
  return mode === 'low' ? 15 : mode === 'normal' ? Math.min(30, userFps) : userFps;
}
