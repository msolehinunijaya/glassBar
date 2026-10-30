// Pure helpers for Quick Settings' Keep awake, Microphone and Bluetooth battery (tested in
// extras.test.ts).
import type { AwakeState, UsingApp } from '../../lib/ipc';

/** The Keep awake tile's second line. */
export function awakeText(s: AwakeState, now: number): string {
  if (!s.on) return 'Off';
  if (s.until == null) return 'On · until you turn it off';
  const m = Math.max(1, Math.ceil((s.until - now) / 60_000));
  return m >= 60 ? `On · ${Math.floor(m / 60)}:${String(m % 60).padStart(2, '0')} left` : `On · ${m}m left`;
}

export type MicOnDock = 'auto' | 'always' | 'never';
/** What the dock shows for the mic: muted (red), live (in use: orange dot), idle (always mode),
 *  or nothing. A muted mic always wins: it must never be forgotten. */
export function micDock(mode: MicOnDock, muted: boolean, inUse: boolean): 'muted' | 'live' | 'idle' | null {
  if (mode === 'never') return null;
  if (muted) return 'muted';
  if (inUse) return 'live';
  return mode === 'always' ? 'idle' : null;
}

/** "Zoom", "Zoom +2". */
export const usingText = (apps: UsingApp[]) => (!apps.length ? '' : apps.length === 1 ? apps[0].name : `${apps[0].name} +${apps.length - 1}`);

/** Battery bar colour: amber at 20 % and below, red at 10 % and below. */
export const levelTone = (p: number): 'ok' | 'low' | 'critical' => (p <= 10 ? 'critical' : p <= 20 ? 'low' : 'ok');
