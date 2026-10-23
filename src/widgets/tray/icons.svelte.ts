import { ipc, type TrayIcon } from '../../lib/ipc';
import { live } from '../../lib/live.svelte';

export const liveTray = () => live<TrayIcon[]>('tray', ipc.trayIcons, []);

/** Which icons sit on the dock and which go to the overflow panel. */
export function split(icons: TrayIcon[], o: { showPromoted: boolean; compact: boolean }) {
  const inline = o.showPromoted ? icons.filter((i) => i.promoted) : [];
  const max = o.compact ? 4 : 10;
  return { inline: inline.slice(0, max), overflow: [...inline.slice(max), ...icons.filter((i) => !inline.includes(i))] };
}
