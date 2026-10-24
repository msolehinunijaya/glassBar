/** Share of to-dos done (0..1), or null when there are none (ring hidden). */
export function progress(todos: { done: boolean }[]): number | null {
  if (!todos.length) return null;
  return todos.filter((t) => t.done).length / todos.length;
}
