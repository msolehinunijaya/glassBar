// Draws one Equalizer frame on a canvas whose transform already maps 1 unit = 1 CSS px, with x
// along the widget's length (L) and y across its thickness (T). Every look shares one colour
// scheme, the glow pass and the reflection; each look only knows how to put its shapes down.
import { mix } from './eq';
import type { EqSettings } from './settings';

export interface Frame { cur: number[]; peak: number[]; quiet: boolean; now: number }
type Paint = string | CanvasGradient;

/** Looks that already fill the whole height, so they skip the reflection. */
const NO_REFLECTION = new Set(['mirror', 'radial', 'particles']);

export function paint(ctx: CanvasRenderingContext2D, s: EqSettings, f: Frame, L: number, T: number, accent: string, album: [string, string] | null) {
  const { cur, peak, now } = f;
  const n = cur.length;
  // Any CSS colour -> '#rrggbb' (the canvas normalises it) so colours can be blended.
  const hex = (c: string) => { ctx.fillStyle = c || '#ffffff'; return String(ctx.fillStyle); };

  // Background goes in last, behind everything (so the reflection fade can't erase it).
  const bg = () => {
    if (s.bgOpacity <= 0) return;
    ctx.save();
    ctx.globalCompositeOperation = 'destination-over';
    ctx.fillStyle = `rgba(0,0,0,${s.bgOpacity / 100})`;
    ctx.beginPath(); ctx.roundRect(0, 0, L, T, Math.min(8, T / 2)); ctx.fill();
    ctx.restore();
  };

  const refl = s.reflection && !NO_REFLECTION.has(s.style);
  const Hall = (T * s.fill) / 100, y0 = (T - Hall) / 2;
  const H = refl ? Hall * (1 - s.reflSize / 100) : Hall; // bars above, reflection below
  const base = y0 + H;
  const gap = Math.max(0, Math.min(s.gap, L / n - 0.5));
  const bw = Math.max(0.5, (L - gap * (n - 1)) / n);
  const xAt = (i: number) => i * (bw + gap);
  const mirror = s.style === 'mirror';

  // ---- colours: per bar (across) or one gradient bottom -> top (up) ----
  const hue0 = ((now / 1000) * s.hueSpeed) % 360;
  const acc = hex(accent);
  const [c1, c2] =
    s.colorMode === 'accent' ? [acc, mix(acc, '#ffffff', 0.55)]
    : s.colorMode === 'album' ? (album ?? [acc, mix(acc, '#ffffff', 0.55)])
    : [hex(s.color1), hex(s.color2)];
  const at = (p: number) =>
    s.colorMode === 'rainbow' ? `hsl(${hue0 + p * 300},100%,55%)` : s.colorMode === 'solid' ? c1 : mix(c1, c2, p);
  const stops = (g: CanvasGradient, mirrored = false) => {
    for (const p of s.colorMode === 'rainbow' ? [0, 0.25, 0.5, 0.75, 1] : [0, 1]) {
      if (mirrored) { g.addColorStop(0.5 + p / 2, at(p)); g.addColorStop(0.5 - p / 2, at(p)); } else g.addColorStop(p, at(p));
    }
    return g;
  };
  const up = s.direction === 'up' && s.colorMode !== 'solid';
  const upGrad = up ? stops(ctx.createLinearGradient(0, base, 0, y0), mirror) : null;
  const fillOf = (i: number): Paint => upGrad ?? at(n > 1 ? i / (n - 1) : 0);
  // One paint for a whole shape (line, area): across = a left-to-right gradient.
  const whole = (): Paint => upGrad ?? (s.colorMode === 'solid' ? c1 : stops(ctx.createLinearGradient(0, 0, L, 0)));

  // ---- idle: breathing dots or a flat line ----
  if (f.quiet) {
    if (s.idle === 'hide' || s.idleOpacity === 0) return;
    const a = s.idleOpacity / 100;
    if (s.style === 'radial') {
      const { cx, cy, r0 } = ring();
      for (let i = 0; i < n; i++) {
        const t = (i / n) * Math.PI * 2 - Math.PI / 2;
        ctx.globalAlpha = s.idle === 'line' ? a : a * (0.55 + 0.45 * Math.sin(now / 380 - i * 0.45));
        ctx.fillStyle = fillOf(i);
        ctx.fillRect(cx + Math.cos(t) * r0 - 0.75, cy + Math.sin(t) * r0 - 0.75, 1.5, 1.5);
      }
    } else {
      const mid = mirror || s.idle === 'line' || s.style === 'particles' ? y0 + Hall / 2 : base - 1.5;
      for (let i = 0; i < n; i++) {
        ctx.fillStyle = fillOf(i);
        if (s.idle === 'line') { ctx.globalAlpha = a; ctx.fillRect(xAt(i), mid - 0.75, bw + gap, 1.5); continue; }
        ctx.globalAlpha = a * (0.55 + 0.45 * Math.sin(now / 380 - i * 0.45));
        ctx.beginPath(); ctx.arc(xAt(i) + bw / 2, mid, Math.max(0.75, Math.min(1.6, bw / 2)), 0, Math.PI * 2); ctx.fill();
      }
    }
    ctx.globalAlpha = 1;
    return bg();
  }

  // ---- the looks. `over` replaces the colours (custom glow colour); `glowPass` skips details. ----
  function ring() {
    const R = Math.min(L, T) / 2 - 1;
    return { cx: L / 2, cy: T / 2, R, r0: R * 0.42 };
  }

  function bars(over: Paint | null, thin = s.style === 'glow', k = 1) {
    for (let i = 0; i < n; i++) {
      const hh = Math.min(H, Math.max(1.5, cur[i] * H * k));
      const lw = thin ? Math.max(1, Math.min(2.5, bw * 0.45)) : bw;
      const x = xAt(i) + (bw - lw) / 2;
      ctx.fillStyle = over ?? fillOf(i);
      ctx.beginPath(); ctx.roundRect(x, mirror ? y0 + (H - hh) / 2 : base - hh, lw, hh, Math.min(s.radius, lw / 2, hh / 2)); ctx.fill();
    }
    if (!s.peaks || k !== 1) return;
    for (let i = 0; i < n; i++) {
      if (peak[i] < 0.02) continue;
      const p = peak[i] * H;
      ctx.fillStyle = over ?? fillOf(i);
      if (mirror) { ctx.fillRect(xAt(i), y0 + (H - p) / 2 - 2, bw, 2); ctx.fillRect(xAt(i), y0 + (H + p) / 2, bw, 2); }
      else ctx.fillRect(xAt(i), Math.max(y0, base - p - 3), bw, 2);
    }
  }

  function led(over: Paint | null, glowPass: boolean) {
    const seg = Math.max(1, s.ledSize), sg = 1;
    const count = Math.max(1, Math.floor((H + sg) / (seg + sg)));
    const ga = ctx.globalAlpha;
    for (let i = 0; i < n; i++) {
      const lit = Math.round(cur[i] * count), top = s.peaks ? Math.round(peak[i] * count) - 1 : -1;
      ctx.fillStyle = over ?? fillOf(i);
      for (let k = 0; k < count; k++) {
        const on = k < lit || (k === top && k > 0);
        if (!on && glowPass) continue;
        ctx.globalAlpha = on ? ga : ga * 0.1; // unlit segments stay faintly visible
        ctx.beginPath(); ctx.roundRect(xAt(i), base - (k + 1) * seg - k * sg, bw, seg, Math.min(s.radius, seg / 2, bw / 2)); ctx.fill();
      }
    }
    ctx.globalAlpha = ga;
  }

  function curve() {
    ctx.beginPath();
    const pt = (i: number) => [xAt(i) + bw / 2, base - Math.max(0.01, cur[i]) * H] as const;
    const [x0, yy0] = pt(0);
    ctx.moveTo(x0, yy0);
    for (let i = 0; i < n - 1; i++) {
      const [x, y] = pt(i), [nx, ny] = pt(i + 1);
      ctx.quadraticCurveTo(x, y, (x + nx) / 2, (y + ny) / 2);
    }
    ctx.lineTo(...pt(n - 1));
    return [x0, xAt(n - 1) + bw / 2];
  }

  function line(over: Paint | null, glowPass: boolean) {
    const p = over ?? whole();
    if (s.area && !glowPass) {
      const [xa, xb] = curve();
      ctx.lineTo(xb, base); ctx.lineTo(xa, base); ctx.closePath();
      const ga = ctx.globalAlpha;
      ctx.globalAlpha = ga * 0.3; ctx.fillStyle = p; ctx.fill(); ctx.globalAlpha = ga;
    }
    curve();
    ctx.lineWidth = s.lineWidth; ctx.lineJoin = 'round'; ctx.lineCap = 'round'; ctx.strokeStyle = p; ctx.stroke();
    if (s.peaks && !glowPass) {
      for (let i = 0; i < n; i++) {
        if (peak[i] < 0.02) continue;
        ctx.fillStyle = over ?? fillOf(i);
        ctx.beginPath(); ctx.arc(xAt(i) + bw / 2, base - peak[i] * H - s.lineWidth, Math.max(1, s.lineWidth * 0.6), 0, Math.PI * 2); ctx.fill();
      }
    }
  }

  function particles(over: Paint | null) {
    const layers = Math.max(1, Math.round(s.layers)), mid = (layers - 1) / 2;
    const dots = Math.max(8, Math.min(400, Math.round(L / 3.5)));
    const r = Math.max(0.6, Math.min(1.4, T / 30));
    const val = (x: number) => { // cur at a fractional bar index
      const c = Math.max(0, Math.min(n - 1, x)), k = Math.floor(c), t = c - k;
      return cur[k] * (1 - t) + cur[Math.min(k + 1, n - 1)] * t;
    };
    const ga = ctx.globalAlpha;
    for (let k = 0; k < layers; k++) {
      const off = k - mid;
      const y = y0 + Hall / 2 + (layers > 1 ? (off / layers) * Hall * 0.5 : 0);
      const damp = 1 - Math.abs(off) * 0.12;
      ctx.globalAlpha = ga * (1 - Math.abs(off) / (layers + 1));
      if (up && !over) ctx.fillStyle = at(layers > 1 ? k / (layers - 1) : 0);
      for (let j = 0; j < dots; j++) {
        const p = j / (dots - 1);
        const v = val(p * (n - 1) + off * 0.8) * damp;
        const wob = Math.sin(p * 12 + now / 600 + k) * 0.05;
        const yy = Math.max(y0, Math.min(y0 + Hall, y - (v * 0.6 + wob) * Hall * 0.5));
        if (over) ctx.fillStyle = over; else if (!up) ctx.fillStyle = at(p);
        ctx.fillRect(p * (L - 2 * r), yy - r, r * 2, r * 2);
      }
    }
    ctx.globalAlpha = ga;
  }

  function radial(over: Paint | null, glowPass: boolean) {
    const { cx, cy, R, r0 } = ring();
    const rg = up ? stops(ctx.createRadialGradient(cx, cy, r0, cx, cy, R)) : null;
    ctx.lineWidth = Math.max(1, ((2 * Math.PI * r0) / n) * 0.6);
    ctx.lineCap = s.radius > 0 ? 'round' : 'butt';
    for (let i = 0; i < n; i++) {
      const t = (i / n) * Math.PI * 2 - Math.PI / 2, c = Math.cos(t), sn = Math.sin(t);
      const len = Math.max(1, cur[i] * (R - r0));
      ctx.strokeStyle = over ?? rg ?? at(i / n);
      ctx.beginPath(); ctx.moveTo(cx + c * r0, cy + sn * r0); ctx.lineTo(cx + c * (r0 + len), cy + sn * (r0 + len)); ctx.stroke();
      if (s.peaks && !glowPass && peak[i] > 0.02) {
        const pr = r0 + peak[i] * (R - r0) + 1.5;
        ctx.fillStyle = over ?? rg ?? at(i / n);
        ctx.fillRect(cx + c * pr - 0.75, cy + sn * pr - 0.75, 1.5, 1.5);
      }
    }
  }

  const look = (over: Paint | null, glowPass: boolean) => {
    switch (s.style) {
      case 'led': return led(over, glowPass);
      case 'line': return line(over, glowPass);
      case 'particles': return particles(over);
      case 'radial': return radial(over, glowPass);
      default: return bars(over);
    }
  };

  const scene = (a: number) => {
    const alpha = (s.barOpacity / 100) * a;
    if (s.style === 'glow') {
      // The tall soft haze above each spike.
      ctx.save(); ctx.filter = 'blur(4px)'; ctx.globalAlpha = 0.18 * alpha; bars(null, false, 3); ctx.restore();
    }
    if (s.glow) {
      const gc = s.glowColor === 'custom' ? hex(s.glowCustom) : null;
      ctx.save(); ctx.filter = `blur(${s.glowSize}px)`; ctx.globalAlpha = (s.glowStrength / 100) * alpha; look(gc, true); ctx.restore();
    }
    ctx.globalAlpha = alpha;
    look(null, false);
    ctx.globalAlpha = 1;
  };

  scene(1);
  if (refl) {
    // Flip the scene under the baseline, then fade it out towards the bottom.
    ctx.save();
    ctx.beginPath(); ctx.rect(0, base, L, T - base); ctx.clip();
    ctx.translate(0, 2 * base); ctx.scale(1, -1);
    scene(s.reflStrength / 100);
    ctx.restore();
    ctx.save();
    ctx.globalCompositeOperation = 'destination-out';
    const g = ctx.createLinearGradient(0, base, 0, y0 + Hall);
    g.addColorStop(0, 'rgba(0,0,0,0)'); g.addColorStop(1, 'rgba(0,0,0,1)');
    ctx.fillStyle = g; ctx.fillRect(0, base, L, T - base);
    ctx.restore();
  }
  bg();
}
