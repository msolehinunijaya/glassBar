// Countdown presets: Malaysian holidays and festivals. Hijri ones use the Umm al-Qura
// calculation (JAKIM's moon sighting may differ by a day, so they are marked an estimate).
import { nextCal, nextYearly, ymd, type Countdown } from './dates';

export interface Preset { key: string; name: string; emoji: string; repeat: Countdown['repeat']; rule?: { m: number; d: number }; mmdd?: string }

export const PRESETS: Preset[] = [
  { key: 'raya', name: 'Hari Raya Aidilfitri', emoji: '🌙', repeat: 'hijri', rule: { m: 10, d: 1 } },
  { key: 'ramadan', name: 'Ramadan', emoji: '🕌', repeat: 'hijri', rule: { m: 9, d: 1 } },
  { key: 'haji', name: 'Hari Raya Haji', emoji: '🐐', repeat: 'hijri', rule: { m: 12, d: 10 } },
  { key: 'muharram', name: 'Awal Muharram', emoji: '🌒', repeat: 'hijri', rule: { m: 1, d: 1 } },
  { key: 'maulid', name: 'Maulidur Rasul', emoji: '✨', repeat: 'hijri', rule: { m: 3, d: 12 } },
  { key: 'merdeka', name: 'Merdeka', emoji: '🇲🇾', repeat: 'yearly', mmdd: '08-31' },
  { key: 'malaysia', name: 'Hari Malaysia', emoji: '🇲🇾', repeat: 'yearly', mmdd: '09-16' },
  { key: 'cny', name: 'Tahun Baru Cina', emoji: '🧧', repeat: 'chinese', rule: { m: 1, d: 1 } },
  { key: 'christmas', name: 'Christmas', emoji: '🎄', repeat: 'yearly', mmdd: '12-25' },
  { key: 'newyear', name: 'New Year', emoji: '🎆', repeat: 'yearly', mmdd: '01-01' },
];

/** Emoji to choose from in the editor. */
export const EMOJI = ['🎯', '🎉', '🎂', '✈️', '📅', '💼', '🎓', '❤️', '🌙', '🏁'];

/** A new countdown from a preset, with its next date from `now`. */
export function fromPreset(p: Preset, now: Date): Countdown {
  const d = p.repeat === 'yearly' ? nextYearly(p.mmdd!, now)
    : nextCal(p.repeat === 'hijri' ? 'islamic-umalqura' : 'chinese', p.rule!.m, p.rule!.d, now);
  return { id: `${p.key}-${now.getTime().toString(36)}`, name: p.name, date: ymd(d), time: null, emoji: p.emoji,
    repeat: p.repeat, rule: p.rule ?? null, created: now.getTime(), remindDayBefore: false };
}

/** Hijri and Chinese dates are calculated: say so next to the date. */
export const isEstimate = (c: Countdown) => c.repeat === 'hijri' || c.repeat === 'chinese';
