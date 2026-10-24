// Generic "load from Rust, reload on <name>:changed" state for widgets.
import { events } from './ipc';

export function live<T>(name: string, load: () => Promise<T>, initial: T) {
  const s = $state<{ v: T }>({ v: initial });
  $effect(() => {
    const run = () => load().then((v) => { s.v = v; }).catch(() => {});
    run();
    const off = events.changed(name, run);
    return () => { off.then((f) => f()); };
  });
  return s;
}
