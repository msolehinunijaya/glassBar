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

/** How far an icon `d` px from the pointer slides away from it while magnified, so the
 *  grown icons don't overlap (macOS-style). It is the running total of the extra size along
 *  the wave, so the icon under the pointer stays put and icons past the wave all move by
 *  (scale - 1) x reach / 2. */
export function spread(d: number, reach: number, scale: number): number {
  const f = Math.abs(d) >= reach ? Math.sign(d) * reach / 2 : d / 2 + (reach / (2 * Math.PI)) * Math.sin((Math.PI * d) / reach);
  return (scale - 1) * f;
}


/** Is an icon (its left/right edge) inside the visible part of a dock `width` wide? */
export const onScreen = (r: { left: number; right: number }, width: number) => r.left >= 0 && r.right <= width;
