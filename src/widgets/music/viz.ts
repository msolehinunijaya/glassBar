/** Rolling level history for the visualizer (newest on the right). */
export function pushPeak(h: number[], p: number, n: number): number[] {
  const next = [...h, Math.max(0, Math.min(1, p))];
  return next.length > n ? next.slice(next.length - n) : next;
}

/** Peak 0..1 -> bar height in px. sqrt-ish curve so quiet passages still move; 2 px floor. */
export function barHeights(h: number[], max: number): number[] {
  return h.map((p) => Math.max(2, Math.round(Math.pow(p, 0.6) * max)));
}
