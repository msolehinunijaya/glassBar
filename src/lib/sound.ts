// Alert sounds. The chime is synthesized, so no audio file ships with the app.
let ctx: AudioContext | null = null;

export function chime() {
  ctx ??= new AudioContext();
  const t = ctx.currentTime;
  for (const [freq, at] of [[880, 0], [1318.5, 0.18]] as const) {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = 'sine';
    osc.frequency.value = freq;
    gain.gain.setValueAtTime(0, t + at);
    gain.gain.linearRampToValueAtTime(0.25, t + at + 0.02);
    gain.gain.exponentialRampToValueAtTime(0.001, t + at + 0.45);
    osc.connect(gain).connect(ctx.destination);
    osc.start(t + at);
    osc.stop(t + at + 0.5);
  }
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
