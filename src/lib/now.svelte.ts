// A "now" that updates once a second, but only while `active()` is true,
// so an idle widget costs nothing.
export function tickingNow(active: () => boolean, everyMs = 1000) {
  const s = $state({ now: Date.now() });
  $effect(() => {
    if (!active()) return;
    const t = setInterval(() => { s.now = Date.now(); }, everyMs);
    s.now = Date.now();
    return () => clearInterval(t);
  });
  return s;
}
