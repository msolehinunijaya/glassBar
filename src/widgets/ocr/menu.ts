import type { CaptureTab } from './index';

/** Popup key of a Capture button's hover menu: "__capmenu:<tab>:<widget instance>". */
export const CAP_MENU = '__capmenu:';
export function parseCapMenu(key: string | null): { tab: CaptureTab; instance: string } | null {
  if (!key?.startsWith(CAP_MENU)) return null;
  const rest = key.slice(CAP_MENU.length);
  const i = rest.indexOf(':');
  const tab = rest.slice(0, i) as CaptureTab;
  return i > 0 && ['shot', 'record', 'ocr', 'clip'].includes(tab) ? { tab, instance: rest.slice(i + 1) } : null;
}
