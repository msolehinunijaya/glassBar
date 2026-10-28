// Mac-style dock magnification (pure, unit tested).

/** 1 at distance 0, easing smoothly to 0 at `reach` and beyond (half a cosine wave). */
export const bell = (d: number, reach: number) =>
  Math.abs(d) >= reach ? 0 : (1 + Math.cos((Math.PI * d) / reach)) / 2;

/** Scale for each icon centre given the pointer position along the dock (null = not on it).
 *  The wave reaches about 2.5 icons to each side. */
export function magnifyScales(centers: number[], pointer: number | null, iconPx: number, scale: number): number[] {
  if (pointer === null) return centers.map(() => 1);
  return centers.map((c) => 1 + (scale - 1) * bell(c - pointer, 2.5 * iconPx));
}

/** How much longer the row gets when each slot of `slotPx` is scaled. */
export const extraLength = (scales: number[], slotPx: number) => scales.reduce((s, k) => s + (k - 1) * slotPx, 0);
