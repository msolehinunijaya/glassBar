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

const toHex = (r: number, g: number, b: number) => '#' + [r, g, b].map((v) => Math.round(v).toString(16).padStart(2, '0')).join('');

/** Two vivid colours from album-art pixels (RGBA): the strongest hue, then the strongest hue
 *  at least 60° away (or a lighter shade of the first). null if the cover is grey/dark. */
export function palette(px: Uint8ClampedArray | number[]): [string, string] | null {
  const B = 12; // 30° hue buckets
  const sum = Array.from({ length: B }, () => ({ w: 0, r: 0, g: 0, b: 0 }));
  for (let i = 0; i + 3 < px.length; i += 4) {
    const [r, g, b] = [px[i], px[i + 1], px[i + 2]];
    const max = Math.max(r, g, b), min = Math.min(r, g, b);
    const sat = max ? (max - min) / max : 0, val = max / 255;
    if (sat < 0.25 || val < 0.2) continue;
    const d = max - min;
    const h = max === r ? ((g - b) / d + 6) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4; // 0..6
    const k = Math.floor((h / 6) * B) % B, w = sat * val;
    sum[k].w += w; sum[k].r += r * w; sum[k].g += g * w; sum[k].b += b * w;
  }
  const order = sum.map((s, k) => ({ ...s, k })).filter((s) => s.w > 0).sort((a, b) => b.w - a.w);
  if (!order.length) return null;
  const col = (s: (typeof order)[number]) => toHex(s.r / s.w, s.g / s.w, s.b / s.w);
  const first = order[0];
  const far = order.find((s) => Math.min(Math.abs(s.k - first.k), B - Math.abs(s.k - first.k)) >= 2 && s.w > first.w * 0.08);
  const c1 = col(first);
  return [c1, far ? col(far) : toHex(...(rgb(c1).map((v) => v + (255 - v) * 0.5) as [number, number, number]))];
}
