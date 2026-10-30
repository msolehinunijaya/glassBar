// Colour picker formats and history (tested in color.test.ts).
export type ColorFormat = 'hex' | 'rgb' | 'hsl';

const h2 = (n: number) => Math.round(n).toString(16).padStart(2, '0').toUpperCase();
export const toHex = (r: number, g: number, b: number) => `#${h2(r)}${h2(g)}${h2(b)}`;
export const rgbOf = (hex: string): [number, number, number] => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16)) as [number, number, number];

function hsl(hex: string): [number, number, number] {
  const [r, g, b] = rgbOf(hex).map((v) => v / 255);
  const max = Math.max(r, g, b), min = Math.min(r, g, b), l = (max + min) / 2, d = max - min;
  if (!d) return [0, 0, Math.round(l * 100)];
  const s = d / (1 - Math.abs(2 * l - 1));
  const h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return [Math.round((h * 60 + 360) % 360), Math.round(s * 100), Math.round(l * 100)];
}

/** "#3B82F6", "rgb(59, 130, 246)" or "hsl(217, 91%, 60%)". */
export function formatColor(hex: string, f: ColorFormat): string {
  if (f === 'rgb') return `rgb(${rgbOf(hex).join(', ')})`;
  if (f === 'hsl') { const [h, s, l] = hsl(hex); return `hsl(${h}, ${s}%, ${l}%)`; }
  return hex.toUpperCase();
}

/** History: newest first, each colour once, at most `max`. */
export const pushColor = (list: string[], hex: string, max = 16) => [hex, ...list.filter((c) => c !== hex)].slice(0, max);

/** Black or white text, whichever reads on this colour. */
export function textOn(hex: string): string {
  const [r, g, b] = rgbOf(hex);
  return 0.299 * r + 0.587 * g + 0.114 * b > 150 ? '#000' : '#fff';
}
