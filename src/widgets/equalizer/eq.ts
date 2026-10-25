// Pure helpers for the Equalizer: map the backend's 96 log bands (20 Hz–20 kHz) onto the
// user's band count and range, smooth them, and pick colours.

/** Must match BANDS / F_LO / F_HI in src-tauri/src/widgets/spectrum.rs. */
export const SRC_BANDS = 96;
const F_LO = 20, F_HI = 20000;

/** Fractional source-band index of a frequency. */
const idx = (hz: number) => (SRC_BANDS * Math.log(hz / F_LO)) / Math.log(F_HI / F_LO);

/** Resample source bands to `n` bars between lowHz and highHz. Bars wider than a source band
 *  take the loudest one inside; narrower bars interpolate so neighbours don't copy each other.
 *  tilt -1..1 lifts treble (+) or bass (−), up to 2x at the far end. */
export function resample(src: number[], n: number, lowHz: number, highHz: number, spacing: 'log' | 'linear', tilt = 0): number[] {
  if (!src.length) return Array(n).fill(0);
  const hzAt = (t: number) => (spacing === 'log' ? lowHz * Math.pow(highHz / lowHz, t) : lowHz + (highHz - lowHz) * t);
  const out: number[] = [];
  for (let j = 0; j < n; j++) {
    const a = idx(hzAt(j / n)), b = idx(hzAt((j + 1) / n));
    let v: number;
    if (b - a < 1) {
      const c = Math.min(Math.max((a + b) / 2 - 0.5, 0), src.length - 1);
      const k = Math.floor(c), t = c - k;
      v = src[k] * (1 - t) + src[Math.min(k + 1, src.length - 1)] * t;
    } else {
      v = 0;
      for (let k = Math.max(0, Math.floor(a)); k < Math.min(src.length, Math.ceil(b)); k++) v = Math.max(v, src[k]);
    }
    const pos = n > 1 ? j / (n - 1) : 0.5;
    out.push(v * Math.pow(2, tilt * (pos - 0.5) * 2));
  }
  return out;
}

/** One animation step. Bars jump up fast and fall at `fall` heights/second; peak caps hold
 *  for `holdMs`, then drop at 1 height/second. Mutates and returns `s`. */
export function step(
  s: { cur: number[]; peak: number[]; hold: number[] },
  target: number[], dtMs: number, nowMs: number, fall: number, holdMs: number,
): typeof s {
  const n = target.length, dt = dtMs / 1000;
  if (s.cur.length !== n) { s.cur = Array(n).fill(0); s.peak = Array(n).fill(0); s.hold = Array(n).fill(0); }
  for (let i = 0; i < n; i++) {
    const t = Math.min(1, Math.max(0, target[i]));
    s.cur[i] = t > s.cur[i] ? s.cur[i] + (t - s.cur[i]) * Math.min(1, dt * 30) : Math.max(t, s.cur[i] - fall * dt);
    if (s.cur[i] >= s.peak[i]) { s.peak[i] = s.cur[i]; s.hold[i] = nowMs + holdMs; }
    else if (nowMs > s.hold[i]) s.peak[i] = Math.max(s.cur[i], s.peak[i] - dt);
  }
  return s;
}

/** '#rrggbb' -> [r, g, b]; anything else (e.g. a CSS colour we can't parse) -> white. */
export function rgb(hex: string): [number, number, number] {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
  if (!m) return [255, 255, 255];
  const v = parseInt(m[1], 16);
  return [v >> 16, (v >> 8) & 255, v & 255];
}

/** Blend two hex colours, t 0..1. */
export const mix = (a: string, b: string, t: number) => {
  const [x, y] = [rgb(a), rgb(b)];
  return `rgb(${x.map((v, i) => Math.round(v + (y[i] - v) * t)).join(',')})`;
};
