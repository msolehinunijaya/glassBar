// Water widget settings. Rust reads the same keys for the goal and the reminders
// (src-tauri/src/widgets/water.rs `cfg`), so names and defaults must match there.
import type { WaterUnit } from './units';

export interface WaterSettings {
  /** fixed: `goal` ml a day. dynamic: 35 ml per kg of `weight`, +10 % on a hot day (hotBonus). */
  goalMode: 'fixed' | 'dynamic'; goal: number; weight: number; hotBonus: boolean;
  /** The amount "+" logs, in ml; also the dock alert's button. Text, as choice buttons save it. */
  cup: '150' | '200' | '250' | '330' | '500';
  /** Remind `every` minutes after the last drink, between fromHour and toHour. */
  remind: boolean; every: number; fromHour: number; toHour: number;
  unit: WaterUnit;
  /** What the dock shows beside the drop. */
  show: 'amount' | 'percent';
}

export const WATER_DEFAULTS: WaterSettings = {
  goalMode: 'fixed', goal: 2000, weight: 60, hotBonus: true, cup: '250',
  remind: true, every: 60, fromHour: 8, toHour: 22, unit: 'ml', show: 'amount',
};
