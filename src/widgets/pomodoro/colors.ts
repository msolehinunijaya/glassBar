// Phase colours and the text on them. One colour per phase drives the whole card; the
// light theme gets a darker shade so white text (and the colour as text) reaches 4.5:1.
// styles/glass.css holds the resulting tokens; colors.test.ts checks they match.
import type { Phase } from '../../lib/ipc';

export const PHASE_HEX: Record<Phase, string> = { work: '#FF6B57', short: '#3DD68C', long: '#5AA9FF' };
export const DARK_INK = '#1c1c1e';
export const LIGHT_INK = '#ffffff';

const rgb = (hex: string) => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
const hexOf = (c: number[]) => '#' + c.map((v) => Math.round(v).toString(16).padStart(2, '0')).join('').toUpperCase();

/** WCAG relative luminance. */
export function luminance(hex: string) {
  const [r, g, b] = rgb(hex).map((v) => { const s = v / 255; return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4; });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}
export function contrast(a: string, b: string) {
  const [x, y] = [luminance(a), luminance(b)].sort((m, n) => n - m);
  return (x + 0.05) / (y + 0.05);
}

/** `hex` darkened (towards black, 1 % steps) until `ink` on it reaches `min`. */
export function shadeFor(hex: string, ink: string, min = 4.5) {
  const c = rgb(hex);
  for (let k = 1; k >= 0; k -= 0.01) {
    const s = hexOf(c.map((v) => v * k));
    if (contrast(s, ink) >= min) return s;
  }
  return '#000000';
}

/** The shade and text colour of a phase in a theme: dark keeps the bright colour with dark
 *  text; light darkens it until white text passes. */
export const phaseTone = (p: Phase, theme: 'light' | 'dark') =>
  theme === 'dark' ? { bg: PHASE_HEX[p], ink: DARK_INK } : { bg: shadeFor(PHASE_HEX[p], LIGHT_INK), ink: LIGHT_INK };
