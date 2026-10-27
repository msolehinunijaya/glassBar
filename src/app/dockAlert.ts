// Dock alerts: for a moment the whole dock says one thing -- the charger moved, the volume or
// brightness changed from outside GlassBar, it is prayer time. One alert at a time, drawn over
// the widget row (which stays mounted and fades). Pure, so the rules can be tested.
import type { ChargerFlip } from '../lib/ipc';
import { FLASH, fillOf, flashText, flashTone } from './chargeFlash';

export type DockAlert =
  | ({ kind: 'charger' } & ChargerFlip)
  | { kind: 'volume'; percent: number; muted: boolean }
  | { kind: 'brightness'; percent: number }
  | { kind: 'prayer'; name: string; at: number };
export type AlertKind = DockAlert['kind'];
export type AlertLength = 'short' | 'normal' | 'long';

/** Settings › Dock alerts. The charger has its own older switch, `chargerFlash`. */
export interface DockAlertsCfg { volume: boolean; brightness: boolean; prayer: boolean; reveal: boolean; length: AlertLength }
export const ALERT_DEFAULTS: DockAlertsCfg = { volume: true, brightness: true, prayer: true, reveal: true, length: 'normal' };

/** Higher wins. Volume and brightness share a rank: the newer one is the one being changed. */
const RANK: Record<AlertKind, number> = { prayer: 3, charger: 2, volume: 1, brightness: 1 };

/**
 * What a new alert does to the one on screen. The same kind updates in place (five presses of
 * volume-up are one alert counting up); a higher or equal one replaces it; a lower one is
 * dropped, not queued -- a volume level from a second ago is not worth showing later.
 */
export function arrive(current: AlertKind | null, next: AlertKind): 'show' | 'update' | 'drop' {
  if (current === null) return 'show';
  if (current === next) return 'update';
  return RANK[next] >= RANK[current] ? 'show' : 'drop';
}

const SCALE: Record<AlertLength, number> = { short: 0.6, normal: 1, long: 1.6 };

/** How long it stays readable after it has arrived (the part hovering pauses), in ms. Volume
 *  and brightness count from the last change. The prayer alert has a button, so it waits. */
export function holdMs(kind: AlertKind, length: AlertLength): number {
  const base = kind === 'prayer' ? 8000 : kind === 'charger' ? FLASH.fill + FLASH.hold : 1200;
  return Math.round(base * SCALE[length]);
}

export interface AlertView {
  icon: 'plug' | 'unplug' | 'volume' | 'mute' | 'sun' | 'prayer';
  /** The big text. */
  value: string;
  label: string;
  /** Any CSS colour. */
  tone: string;
  /** How much of the dock the level fill covers, 0..1, or null for no fill. */
  fill: number | null;
}

const clampPct = (p: number) => Math.max(0, Math.min(100, Math.round(p)));

export function viewOf(a: DockAlert, time = (ms: number) => new Date(ms).toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' })): AlertView {
  switch (a.kind) {
    case 'charger': {
      const t = flashText(a);
      return { icon: a.plugged ? 'plug' : 'unplug', value: t.pct, label: t.label, tone: flashTone(a.plugged, a.percent), fill: fillOf(a.percent) };
    }
    case 'volume': {
      const p = clampPct(a.percent);
      const silent = a.muted || p === 0;
      return { icon: silent ? 'mute' : 'volume', value: a.muted ? 'Muted' : `${p}%`, label: 'Volume', tone: 'var(--accent)', fill: a.muted ? 0 : p / 100 };
    }
    case 'brightness': {
      const p = clampPct(a.percent);
      return { icon: 'sun', value: `${p}%`, label: 'Brightness', tone: 'var(--accent)', fill: p / 100 };
    }
    case 'prayer':
      return { icon: 'prayer', value: a.name, label: `${time(a.at)} · prayer time`, tone: 'var(--accent)', fill: null };
  }
}
