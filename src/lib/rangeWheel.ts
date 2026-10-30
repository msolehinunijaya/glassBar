// The mouse wheel moves a slider (Settings and panels): one step per notch, up = more. It
// only takes the wheel once the pointer has rested on the slider (REST_MS), so scrolling a
// panel past a slider still scrolls the panel. Each notch fires the slider's `input` event (live
// preview); `change` (save) follows once the wheel stops.
export const REST_MS = 200;
const CHANGE_AFTER_MS = 350;

/** Pure: the next value after `notches` (positive = up) on a slider. */
export function wheelStep(value: number, min: number, max: number, step: number, notches: number): number {
  const s = step > 0 ? step : 1;
  const v = Math.min(max, Math.max(min, value + notches * s));
  const d = Math.max(0, (String(s).split('.')[1] ?? '').length);
  return Number(v.toFixed(d));
}

export function installRangeWheel(target: Window = window) {
  let over: HTMLInputElement | null = null;
  let overAt = 0;
  const timers = new WeakMap<HTMLInputElement, ReturnType<typeof setTimeout>>();
  const rangeOf = (t: EventTarget | null) => (t instanceof HTMLInputElement && t.type === 'range' && !t.disabled ? t : null);
  target.addEventListener('pointerover', (e) => { const r = rangeOf(e.target); if (r !== over) { over = r; overAt = performance.now(); } }, { capture: true });
  target.addEventListener('wheel', (e) => {
    const r = rangeOf(e.target);
    if (!r || r !== over || performance.now() - overAt < REST_MS || r.dataset.wheel === 'off') return;
    e.preventDefault();
    const notches = Math.sign(-e.deltaY || e.deltaX);
    if (!notches) return;
    const next = wheelStep(+r.value, +r.min || 0, r.max === '' ? 100 : +r.max, r.step === 'any' ? 1 : +r.step || 1, notches);
    if (next === +r.value) return;
    r.value = String(next);
    r.dispatchEvent(new Event('input', { bubbles: true }));
    clearTimeout(timers.get(r));
    timers.set(r, setTimeout(() => r.dispatchEvent(new Event('change', { bubbles: true })), CHANGE_AFTER_MS));
  }, { passive: false, capture: true });
}
