import { ipc, type TrayIcon } from '../../lib/ipc';
import { live } from '../../lib/live.svelte';

export const liveTray = () => live<TrayIcon[]>('tray', ipc.trayIcons, []);
