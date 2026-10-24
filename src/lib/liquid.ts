// Liquid Glass: the sheen on the card under the pointer follows it (--mx, see glass.css).
// At most one update per frame, and only while the pointer moves, so an idle or hidden
// dock does no work. Off unless the theme is Liquid Glass with full motion.
const root = document.documentElement;
const reduced = matchMedia('(prefers-reduced-motion: reduce)');
let raf = 0;
let last: PointerEvent | null = null;

addEventListener('pointermove', (e) => {
  if (root.dataset.preset !== 'liquid' || root.dataset.motion !== 'full' || reduced.matches) return;
  last = e;
  raf ||= requestAnimationFrame(() => {
    raf = 0;
    const el = (last?.target as Element | null)?.closest?.<HTMLElement>('.pill, .card, .tile, .side, .glass');
    if (!el || !last) return;
    const r = el.getBoundingClientRect();
    el.style.setProperty('--mx', `${Math.round(((last.clientX - r.left) / Math.max(r.width, 1)) * 100)}%`);
  });
}, { passive: true });
