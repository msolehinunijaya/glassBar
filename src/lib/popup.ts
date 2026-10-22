import { ipc } from './ipc';

/** Open the shared popup window next to `el` (Rust positions it beside the bar). */
export function openPopup(instance: string, el: HTMLElement, width: number, height: number) {
  return ipc.popupOpen(instance, el.getBoundingClientRect(), width, height);
}

export const GALLERY = '__gallery';
