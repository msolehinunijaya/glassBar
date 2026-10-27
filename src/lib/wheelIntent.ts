// Who a mouse-wheel gesture belongs to. A gesture is wheel events less than GAP_MS apart, and
// it belongs to whatever it *started* on: a fast scroll along the dock keeps scrolling the
// dock when the pointer passes over a volume control, instead of changing the volume.
// A control only takes a gesture that started on it after the pointer rested there REST_MS.

export const GAP_MS = 500;
export const REST_MS = 250;

interface Box { contains(n: unknown): boolean }

export function wheelIntent(gap = GAP_MS, rest = REST_MS) {
  let lastAt = -Infinity;
  let start: unknown = null;
  let startAt = 0;
  let sideways = false;
  return {
    /** Every wheel event, before any control sees it. */
    seen(target: unknown, now: number, dx: number, dy: number) {
      if (now - lastAt > gap) { start = target; startAt = now; sideways = Math.abs(dx) > Math.abs(dy); }
      lastAt = now;
    },
    /** May `node` (entered by the pointer at `enteredAt`) use the current gesture? */
    owns(node: Box, enteredAt: number): boolean {
      return !sideways && start !== null && node.contains(start) && startAt - enteredAt >= rest;
    },
  };
}

// One tracker per window, fed from the capture phase so it sees each event before the
// control under the pointer does.
const shared = wheelIntent();
if (typeof addEventListener === 'function') {
  addEventListener('wheel', (e) => shared.seen(e.target, performance.now(), e.deltaX, e.deltaY), { capture: true, passive: true });
}

/** For a volume control: call `enter` on pointerenter, and only act on a wheel when `owns(e)`.
 *  When it doesn't, leave the event alone so the dock scrolls. */
export function wheelControl() {
  let enteredAt = Infinity;
  return {
    enter() { enteredAt = performance.now(); },
    owns(e: WheelEvent) { return shared.owns(e.currentTarget as unknown as Box, enteredAt); },
  };
}
