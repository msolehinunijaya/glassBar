// Launch bounce bookkeeping (pure, unit tested): an icon bounces from launch until its app
// shows a window it didn't have before, or for 10 s at most.
export type Bounces = Record<string, { since: number; ids: number[] }>;

export const BOUNCE_MS = 10_000;

/** Start (or restart) bouncing `key`; `ids` = the windows it already had. */
export const startBounce = (b: Bounces, key: string, ids: number[], now: number): Bounces => ({ ...b, [key]: { since: now, ids } });

/** Drop the bounces that are done. */
export function settleBounces(b: Bounces, groups: { key: string; windows: { id: number }[] }[], now: number): Bounces {
  const out: Bounces = {};
  for (const [key, v] of Object.entries(b)) {
    const g = groups.find((x) => x.key === key);
    const opened = g?.windows.some((w) => !v.ids.includes(w.id));
    if (!opened && now - v.since <= BOUNCE_MS) out[key] = v;
  }
  return out;
}
