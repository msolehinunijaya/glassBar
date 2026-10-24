// Pointer drag-to-reorder for the Settings dock preview (a row or a column) and the card
// grid. Same idea as the bar's edit mode: move 6 px to pick an item up, the list reorders
// live under the pointer, Esc puts it back, the new order is saved on release.
// Tracks the pointer on the window, not with pointer capture (reordering moves the DOM node).

export type Axis = 'x' | 'y' | 'grid';

/** Where the dragged item lands among the others: how many of them sit before the pointer. */
export function landingIndex(others: DOMRect[], p: { x: number; y: number }, axis: Axis): number {
  return others.filter((r) => {
    const cx = r.left + r.width / 2, cy = r.top + r.height / 2;
    if (axis === 'x') return cx < p.x;
    if (axis === 'y') return cy < p.y;
    return r.bottom <= p.y || (r.top <= p.y && cx < p.x); // rows above, or left in this row
  }).length;
}

export function dragSort(
  e: PointerEvent,
  id: string,
  ids: string[],
  root: HTMLElement,
  axis: Axis,
  show: (order: string[] | null, dragging: string | null) => void,
  commit: (order: string[]) => void,
) {
  if (e.button !== 0 || (e.target as HTMLElement).closest('button:not(.look), input, select, textarea, a, label')) return;
  const start = { x: e.clientX, y: e.clientY };
  let order: string[] | null = null;
  let raf = 0;
  let last = e;

  const frame = () => {
    raf = 0;
    if (!order) return;
    const others = [...root.querySelectorAll<HTMLElement>('[data-id]')].filter((el) => el.dataset.id !== id).map((el) => el.getBoundingClientRect());
    const to = landingIndex(others, { x: last.clientX, y: last.clientY }, axis);
    const rest = order.filter((x) => x !== id);
    const next = [...rest.slice(0, to), id, ...rest.slice(to)];
    if (next.join() !== order.join()) { order = next; show(order, id); }
  };
  const move = (ev: PointerEvent) => {
    last = ev;
    if (!order && Math.hypot(ev.clientX - start.x, ev.clientY - start.y) > 6) { order = ids.slice(); show(order, id); }
    if (order && !raf) raf = requestAnimationFrame(frame);
  };
  const end = (save: boolean) => {
    cancelAnimationFrame(raf);
    removeEventListener('pointermove', move);
    removeEventListener('pointerup', up);
    removeEventListener('pointercancel', cancel);
    removeEventListener('keydown', key);
    const final = order;
    show(null, null);
    if (save && final && final.join() !== ids.join()) commit(final);
  };
  const up = () => end(true);
  const cancel = () => end(false);
  const key = (k: KeyboardEvent) => { if (k.key === 'Escape') end(false); };
  addEventListener('pointermove', move);
  addEventListener('pointerup', up);
  addEventListener('pointercancel', cancel);
  addEventListener('keydown', key);
}

/** Was this pointerdown turned into a drag? Clicks right after a drag are ignored. */
export const dragged = { at: 0 };
