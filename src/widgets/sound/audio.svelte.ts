// Shared live audio state for the pill and the popup.
import { Volume, Volume1, Volume2, VolumeX } from '@lucide/svelte';
import { events, ipc, type AudioState } from '../../lib/ipc';

export function liveAudio() {
  const s = $state<{ v: AudioState | null }>({ v: null });
  $effect(() => {
    ipc.audioState().then((a) => { s.v = a; }).catch(() => {});
    const off = events.audioChanged((a) => { s.v = a; });
    return () => { off.then((f) => f()); };
  });
  return s;
}

export const volumeIcon = (a: AudioState | null) =>
  !a || a.muted || a.volume === 0 ? VolumeX : a.volume < 0.34 ? Volume : a.volume < 0.67 ? Volume1 : Volume2;

export const pct = (v: number) => `${Math.round(v * 100)}%`;
