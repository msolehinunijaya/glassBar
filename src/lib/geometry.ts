// Pure layout math for the bar (no DOM, no IPC) so it can be unit tested.
export type Edge = 'top' | 'bottom' | 'left' | 'right';
export interface Rect { x: number; y: number; w: number; h: number }

const dist2 = (r: Rect, px: number, py: number) => {
  const dx = Math.max(r.x - px, 0, px - (r.x + r.w));
  const dy = Math.max(r.y - py, 0, py - (r.y + r.h));
  return dx * dx + dy * dy;
};

/** Which monitor the window belongs to and the edge to dock to (null = float).
 *  `hide` = the bar was dragged past that edge (its centre left the screen): auto-hide there. */
export function snap(win: Rect, monitors: Rect[], threshold: number): { monitor: number; edge: Edge | null; hide: boolean } {
  const cx = win.x + win.w / 2, cy = win.y + win.h / 2;
  let monitor = 0;
  monitors.forEach((m, i) => { if (dist2(m, cx, cy) < dist2(monitors[monitor], cx, cy)) monitor = i; });
  const m = monitors[monitor];
  const past: Edge | null = cy > m.y + m.h ? 'bottom' : cy < m.y ? 'top' : cx < m.x ? 'left' : cx > m.x + m.w ? 'right' : null;
  if (past) return { monitor, edge: past, hide: true };
  const gaps: [Edge, number][] = [
    ['top', win.y - m.y],
    ['bottom', m.y + m.h - (win.y + win.h)],
    ['left', win.x - m.x],
    ['right', m.x + m.w - (win.x + win.w)],
  ];
  const [edge, gap] = gaps.reduce((a, b) => (Math.abs(b[1]) < Math.abs(a[1]) ? b : a));
  return { monitor, edge: Math.abs(gap) <= threshold ? edge : null, hide: false };
}

/** Copy of `list` with the item at `from` moved to `to` (clamped). */
export function reorder<T>(list: T[], from: number, to: number): T[] {
  const out = list.slice();
  const [item] = out.splice(from, 1);
  out.splice(Math.max(0, Math.min(to, out.length)), 0, item);
  return out;
}

/** Index of the item whose center is nearest the pointer coordinate. */
export function indexAt(centers: number[], p: number): number {
  let best = 0;
  centers.forEach((c, i) => { if (Math.abs(c - p) < Math.abs(centers[best] - p)) best = i; });
  return best;
}

/** Free drag: where the dragged widget lands = how many of the *other* widgets'
 *  centres lie before the pointer. */
export function insertIndex(otherCenters: number[], p: number): number {
  return otherCenters.filter((c) => c < p).length;
}
