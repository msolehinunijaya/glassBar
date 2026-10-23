// Alert sounds. The chime is synthesized, so no audio file ships with the app.
let ctx: AudioContext | null = null;
let idle: ReturnType<typeof setTimeout>;

export const chime = () => tones([[880, 0], [1318.5, 0.18]], 0.25);

/** Device plugged in: soft rising pair. Removed: falling pair. */
export const deviceTone = (connected: boolean, volume = 0.4) =>
  tones(connected ? [[660, 0], [990, 0.09]] : [[990, 0], [660, 0.09]], 0.2 * Math.max(0, Math.min(1, volume)), 0.22);

async function tones(notes: readonly (readonly [number, number])[], peak: number, len = 0.45) {
  ctx ??= new AudioContext();
  clearTimeout(idle);
  await ctx.resume();
  const t = ctx.currentTime;
  for (const [freq, at] of notes) {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = 'sine';
    osc.frequency.value = freq;
    gain.gain.setValueAtTime(0, t + at);
    gain.gain.linearRampToValueAtTime(Math.max(peak, 0.0011), t + at + 0.02);
    gain.gain.exponentialRampToValueAtTime(0.001, t + at + len);
    osc.connect(gain).connect(ctx.destination);
    osc.start(t + at);
    osc.stop(t + at + len + 0.05);
  }
  // A running AudioContext keeps the audio device (and a render thread) busy.
  idle = setTimeout(() => ctx?.suspend(), 1500);
}

let current: HTMLAudioElement | null = null;

/** Play a user-chosen sound (e.g. azan) given as a data: URL. */
export function playFile(src: string) {
  current?.pause();
  current = new Audio(src);
  current.play().catch(() => chime());
}

export function stopSound() {
  current?.pause();
  current = null;
}
