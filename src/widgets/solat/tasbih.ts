// Tasbih: the dhikr to choose from and the targets of each step. Tasbih Fatimah is three
// phrases in a row (33 · 33 · 34); the others repeat one phrase to the chosen target.
export const DHIKR = [
  { key: 'subhanallah', label: 'Subhanallah' },
  { key: 'alhamdulillah', label: 'Alhamdulillah' },
  { key: 'allahuakbar', label: 'Allahuakbar' },
  { key: 'astaghfirullah', label: 'Astaghfirullah' },
  { key: 'lailahaillallah', label: 'Lailahaillallah' },
  { key: 'selawat', label: 'Selawat' },
  { key: 'tasbih_fatimah', label: 'Tasbih Fatimah' },
] as const;
export type DhikrKey = (typeof DHIKR)[number]['key'];

const FATIMAH = ['Subhanallah', 'Alhamdulillah', 'Allahuakbar'];
export const TARGETS = [33, 99, 100, 1000];

export const stepsOf = (dhikr: string, target: number): number[] => (dhikr === 'tasbih_fatimah' ? [33, 33, 34] : [Math.max(1, target || 33)]);

/** The phrase to say at this step. */
export const phraseOf = (dhikr: string, step: number): string =>
  dhikr === 'tasbih_fatimah' ? FATIMAH[step % 3] : DHIKR.find((d) => d.key === dhikr)?.label ?? 'Subhanallah';
