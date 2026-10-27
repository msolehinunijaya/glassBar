// Is the adzan playing, and stop it -- from any GlassBar window. The sound itself plays in the
// dock window (lib/sound.ts); the prayer panel lives in the popup window, so the state and
// the "stop" travel between them as app events.
import { emit, listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { stopSound } from './sound';

const isBar = getCurrentWindow().label === 'bar';
export const adhan = $state({ playing: false });

if (isBar) {
  listen('adhan:ask', () => { emit('adhan:state', adhan.playing).catch(() => {}); }).catch(() => {});
  listen('adhan:stop', () => stopSound()).catch(() => {});
} else {
  listen<boolean>('adhan:state', (e) => { adhan.playing = e.payload; }).catch(() => {});
  emit('adhan:ask').catch(() => {}); // a panel opened mid-adzan learns it at once
}

/** Dock window only: the adzan started or ended. */
export function setAdhanPlaying(on: boolean) {
  adhan.playing = on;
  emit('adhan:state', on).catch(() => {});
}

/** Stop the adzan (paused music resumes, as when it ends by itself). */
export function stopAdhan() {
  if (isBar) stopSound(); else emit('adhan:stop').catch(() => {});
}
