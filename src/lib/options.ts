/** Flip one checkbox in a group, but never let fewer than `min` stay checked. */
export function toggleCheck<T extends Record<string, boolean>>(values: T, key: keyof T, on: boolean, min: number): T {
  const next = { ...values, [key]: on };
  const count = Object.values(next).filter(Boolean).length;
  return count < min ? values : next;
}
