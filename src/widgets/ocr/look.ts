import type { CaptureTab } from './index';

/** Colour of each capture kind (gradient top → bottom), used for its icon tile. */
export const TINT: Record<CaptureTab, [string, string]> = {
  shot: ['#4aa8ff', '#0a6ae0'],   // Screenshot: blue
  record: ['#ff7a6e', '#e3342b'], // Recording: red
  ocr: ['#cf8bff', '#8b3fd6'],    // OCR Snap: purple
  clip: ['#ffbf4d', '#f08c00'],   // Clipboard: orange
};
export const tile = (k: CaptureTab) => `linear-gradient(160deg, ${TINT[k][0]}, ${TINT[k][1]})`;

/** History fades with age: the newest item at full strength, each older one a step
 *  lighter, down to 40%. Hovering an item shows it at full strength. */
export const fade = (i: number) => Math.max(0.4, 1 - i * 0.1);
