// What goes on the shelf and what falls off it. Pure, so the cap and the ordering can be
// checked without files or a dock.

/** A shelf is a handful of files you are carrying, not a folder. Five is the handful. */
export const MAX = 5;

/** Windows treats paths as case-insensitive, so C:\A.txt and c:\a.txt are one file. */
const same = (p: string) => p.trim().toLowerCase().replace(/[\\/]+$/, '');

/**
 * Put `paths` on the shelf, newest first, and let the oldest fall off the end.
 *
 * A file already on the shelf moves back to the front rather than appearing twice, which is
 * what dropping it again plainly means.
 */
export function add(files: string[], paths: string[]): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const p of [...paths, ...files]) {
    const key = same(p);
    if (!key || seen.has(key)) continue;
    seen.add(key);
    out.push(p.trim());
    if (out.length === MAX) break;
  }
  return out;
}

/** Take one file off the shelf. The file itself is untouched. */
export const remove = (files: string[], path: string): string[] => files.filter((f) => same(f) !== same(path));

/** Take several off at once: what a completed drag-out reports back. */
export function removeAll(files: string[], paths: string[]): string[] {
  const gone = new Set(paths.map(same));
  return files.filter((f) => !gone.has(same(f)));
}

/** The name Windows would show for a path. */
export const nameOf = (p: string): string => p.replace(/[\\/]+$/, '').split(/[\\/]/).pop() || p;

/** The extension, upper-case and without the dot, for the icon's fallback label. */
export const extOf = (p: string): string => {
  const n = nameOf(p);
  const i = n.lastIndexOf('.');
  return i > 0 ? n.slice(i + 1).toUpperCase() : '';
};

/**
 * Is this drop meant for the shelf? `position` from Tauri is in **physical** pixels from the
 * webview's top-left, while a DOM rect is in CSS pixels, so the ratio has to come out --
 * without it the hit box is wrong on any display that isn't at 100 %.
 */
export function hits(rect: { left: number; top: number; right: number; bottom: number }, x: number, y: number, dpr: number): boolean {
  const cx = x / (dpr || 1);
  const cy = y / (dpr || 1);
  return cx >= rect.left && cx <= rect.right && cy >= rect.top && cy <= rect.bottom;
}

/** How many shelved files the dock shows: all of them, or -- collapsed -- only the newest
 *  (the rest behind a "+N"), until the pointer opens it. */
export const shownCount = (n: number, open: boolean, collapse: boolean): number => (!collapse || open || n <= 1 ? n : 1);
