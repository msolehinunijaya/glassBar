/** Flip one checkbox in a group, but never let fewer than `min` stay checked. */
export function toggleCheck<T extends Record<string, boolean>>(values: T, key: keyof T, on: boolean, min: number): T {
  const next = { ...values, [key]: on };
  const count = Object.values(next).filter(Boolean).length;
  return count < min ? values : next;
}

/** A row's identity in an options list: the setting it changes (two rows can share a visible
 *  label, e.g. "Colour" for the colour mode and the colour itself). A duplicate identity makes
 *  the built app fail to draw the whole panel. */
export const optionId = (o: { label: string; key?: string }) => o.key ?? `label:${o.label}`;
