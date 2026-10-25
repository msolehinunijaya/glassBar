// Open a panel by hovering (after a short pause), without taking focus; it closes when the
// pointer leaves both the widget and the panel. The popup window handles the closing.
import { ipc } from './ipc';

/** A "left" whose position is still inside the element is a stray one (a tooltip or a window
 *  appearing under the pointer), not the pointer really leaving. */
export function reallyLeft(e: PointerEvent | undefined, el: HTMLElement | undefined): boolean {
  if (!e || !el) return true;
  const r = el.getBoundingClientRect();
  return !(e.clientX > r.left && e.clientX < r.right && e.clientY > r.top && e.clientY < r.bottom);
}

export function hoverPanel(delay = 350) {
  let t: ReturnType<typeof setTimeout> | undefined;
  let el: HTMLElement | undefined;
  return {
    enter(e: PointerEvent, key: string, width: number, height: number) {
      el = e.currentTarget as HTMLElement;
      const at = el;
      clearTimeout(t);
      t = setTimeout(() => ipc.popupOpen(key, at.getBoundingClientRect(), width, height, false), delay);
    },
    leave(e?: PointerEvent) {
      if (!reallyLeft(e, el)) return;
      clearTimeout(t);
      ipc.popupLeave();
    },
    /** A click or drag: don't open after all. */
    cancel() { clearTimeout(t); },
  };
}
