import type { Station } from '../../lib/ipc';

/** DOE stations grouped by state (both sorted by name); stations with no reading are left out. */
export function statesOf(stations: Station[]): [string, Station[]][] {
  const by = new Map<string, Station[]>();
  for (const s of stations) {
    if (s.api == null) continue;
    by.set(s.state, [...(by.get(s.state) ?? []), s]);
  }
  return [...by.entries()]
    .map(([state, list]) => [state, list.sort((a, b) => a.name.localeCompare(b.name))] as [string, Station[]])
    .sort((a, b) => a[0].localeCompare(b[0]));
}
