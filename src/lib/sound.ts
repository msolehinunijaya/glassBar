// Alert sounds. The chime is synthesized, so no audio file ships with the app.
let ctx: AudioContext | null = null;
let idle: ReturnType<typeof setTimeout>;

export const chime = () => tones([[880, 0], [1318.5, 0.18]], 0.25);

/** Device plugged in: soft rising pair. Removed: falling pair. */
export const deviceTone = (connected: boolean, volume = 0.4) =>
  tones(connected ? [[660, 0], [990, 0.09]] : [[990, 0], [660, 0.09]], 0.2 * Math.max(0, Math.min(1, volume)), 0.22);

/** Camera shutter for screenshots: a short filtered noise burst. Quiet. */
export async function shutter() {
  ctx ??= new AudioContext();
  clearTimeout(idle);
  await ctx.resume();
  const t = ctx.currentTime, len = 0.09;
  const buf = ctx.createBuffer(1, Math.floor(ctx.sampleRate * len), ctx.sampleRate);
  const d = buf.getChannelData(0);
  for (let i = 0; i < d.length; i++) d[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / d.length, 3);
  const src = ctx.createBufferSource(), f = ctx.createBiquadFilter(), g = ctx.createGain();
  src.buffer = buf; f.type = 'bandpass'; f.frequency.value = 2600; g.gain.value = 0.35;
  src.connect(f).connect(g).connect(ctx.destination);
  src.start(t);
  idle = setTimeout(() => ctx?.suspend(), 1500);
}

/** OCR done: a soft high tick. */
export const tick = () => tones([[1568, 0]], 0.12, 0.12);

/** Charger plugged in: bright rising C–E–G. Unplugged: soft falling E–A, lower and rounder. */
export const chargerTone = (plugged: boolean, volume = 0.4) => {
  const v = 0.22 * Math.max(0, Math.min(1, volume));
  return plugged
    ? tones([[1046.5, 0], [1318.5, 0.08], [1568, 0.16]], v, 0.3, 'triangle')
    : tones([[659.3, 0], [440, 0.13]], v, 0.38, 'sine');
};

async function tones(notes: readonly (readonly [number, number])[], peak: number, len = 0.45, wave: OscillatorType = 'sine') {
  ctx ??= new AudioContext();
  clearTimeout(idle);
  await ctx.resume();
  const t = ctx.currentTime;
  for (const [freq, at] of notes) {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = wave;
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

/** Short adzan: the opening takbir of the bundled recording, which ends in a pause at 43.0-43.7 s (ffmpeg silencedetect). */
export const SHORT_ADHAN_S = 43.3;
const FADE_S = 1.5;

/** Play a sound file (the adzan): whole, or with `short` only its opening, faded out. */
export function playFile(src: string, short = false) {
  current?.pause();
  const a = (current = new Audio(src));
  if (short) {
    a.addEventListener('timeupdate', () => {
      const left = SHORT_ADHAN_S - a.currentTime;
      if (left <= 0) { a.pause(); if (current === a) current = null; }
      else if (left < FADE_S) a.volume = Math.max(0, left / FADE_S);
    });
  }
  a.play().catch(() => chime());
}

export function stopSound() {
  current?.pause();
  current = null;
}
