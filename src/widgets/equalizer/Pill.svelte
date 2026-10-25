<script lang="ts">
  // Live spectrum on a canvas. The backend sends 96 bands (~60/s) while this is mounted and the
  // window is visible; each frame resamples them to the user's bar count, smooths and draws.
  // Nothing playing for 0.4 s: bars fall to zero, then the idle look (dots / line / nothing).
  import { listen } from '@tauri-apps/api/event';
  import { ipc } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import type { EqSettings } from './index';
  import { mix, resample, step } from './eq';

  let { settings: s, vertical }: PillProps<EqSettings> = $props();
  let canvas: HTMLCanvasElement;
  let src: number[] = [];
  let lastData = 0;

  $effect(() => {
    ipc.audioSpectrum();
    const lease = setInterval(() => { if (document.visibilityState === 'visible') ipc.audioSpectrum(); }, 2000);
    const off = listen<number[]>('audio:spectrum', (e) => { src = e.payload; lastData = performance.now(); });
    return () => { clearInterval(lease); off.then((f) => f()); };
  });

  $effect(() => {
    const ctx = canvas.getContext('2d')!;
    const st = { cur: [] as number[], peak: [] as number[], hold: [] as number[] };
    let raf = 0, last = 0, quiet = true;
    // Any CSS colour -> '#rrggbb' (the canvas normalises it), so accent vars blend too.
    const hex = (c: string) => { ctx.fillStyle = c || '#ffffff'; return String(ctx.fillStyle); };

    const frame = (now: number) => {
      raf = requestAnimationFrame(frame);
      if (now - last < (quiet ? 66 : 1000 / +s.fps - 2)) return; // idle redraws at ~15 fps
      const dt = last ? Math.min(100, now - last) : 16;
      last = now;

      const dpr = devicePixelRatio || 1, w = canvas.clientWidth, h = canvas.clientHeight;
      if (!w || !h) return;
      if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
        canvas.width = Math.round(w * dpr); canvas.height = Math.round(h * dpr);
      }
      const L = vertical ? h : w, T = vertical ? w : h; // length and thickness in CSS px
      const n = Math.max(1, Math.round(s.bands));
      const signal = now - lastData < 400;
      const target = signal
        ? resample(src, n, +s.lowHz, +s.highHz, s.spacing, s.tilt / 100).map((v) => (v * s.gain) / 100)
        : Array(n).fill(0);
      if (s.flip) target.reverse();
      step(st, target, dt, now, s.fall * 0.5, s.peaks ? s.peakHold : 0);
      quiet = !signal && st.cur.every((v) => v < 0.01) && st.peak.every((v) => v < 0.01);

      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.clearRect(0, 0, w, h);
      if (vertical) { ctx.translate(w, 0); ctx.rotate(Math.PI / 2); } // draw along the length, bars grow right

      if (s.bgOpacity > 0) {
        ctx.fillStyle = `rgba(0,0,0,${s.bgOpacity / 100})`;
        ctx.beginPath(); ctx.roundRect(0, 0, L, T, Math.min(8, T / 2)); ctx.fill();
      }

      const H = (T * s.fill) / 100, y0 = (T - H) / 2;
      const gap = Math.min(s.gap, L / n - 0.5);
      const bw = Math.max(0.5, (L - gap * (n - 1)) / n);
      const xAt = (i: number) => i * (bw + gap);
      const mirror = s.style === 'mirror';

      // Colours: per bar (across) or one vertical gradient shared by all bars (up).
      const hue0 = ((now / 1000) * s.hueSpeed) % 360;
      const accent = hex(getComputedStyle(canvas).getPropertyValue('--accent').trim());
      const [c1, c2] = s.colorMode === 'accent' ? [accent, mix(accent, '#ffffff', 0.55)] : [hex(s.color1), hex(s.color2)];
      const at = (p: number) =>
        s.colorMode === 'rainbow' ? `hsl(${hue0 + p * 300},100%,55%)` : s.colorMode === 'solid' ? c1 : mix(c1, c2, p);
      let up: CanvasGradient | null = null;
      if (s.direction === 'up' && s.colorMode !== 'solid') {
        up = ctx.createLinearGradient(0, y0 + H, 0, y0);
        const stops = s.colorMode === 'rainbow' ? [0, 0.25, 0.5, 0.75, 1] : [0, 1];
        for (const p of stops) {
          const c = at(p);
          if (mirror) { up.addColorStop(0.5 + p / 2, c); up.addColorStop(0.5 - p / 2, c); } else up.addColorStop(p, c);
        }
      }
      const fillOf = (i: number) => up ?? at(n > 1 ? i / (n - 1) : 0);

      // One pass of bars (or caps). k scales height (the glow-spike haze draws them taller).
      const bars = (vals: number[], color: (i: number) => string | CanvasGradient, k = 1, thin = false) => {
        for (let i = 0; i < n; i++) {
          const hh = Math.min(H, Math.max(1.5, vals[i] * H * k));
          const lw = thin ? Math.max(1, Math.min(2.5, bw * 0.45)) : bw;
          const x = xAt(i) + (bw - lw) / 2;
          const y = mirror ? y0 + (H - hh) / 2 : y0 + H - hh;
          ctx.fillStyle = color(i);
          ctx.beginPath(); ctx.roundRect(x, y, lw, hh, Math.min(s.radius, lw / 2, hh / 2)); ctx.fill();
        }
      };
      const draw = (color: (i: number) => string | CanvasGradient) => {
        if (s.style === 'glow') bars(st.cur, color, 1, true);
        else bars(st.cur, color);
        if (s.peaks) {
          for (let i = 0; i < n; i++) {
            if (st.peak[i] < 0.02) continue;
            const p = st.peak[i] * H, ch = 2;
            ctx.fillStyle = color(i);
            if (mirror) {
              ctx.fillRect(xAt(i), y0 + (H - p) / 2 - ch, bw, ch);
              ctx.fillRect(xAt(i), y0 + (H + p) / 2, bw, ch);
            } else ctx.fillRect(xAt(i), Math.max(0, y0 + H - p - ch - 1), bw, ch);
          }
        }
      };

      if (quiet) {
        // Idle: breathing dots or a flat line in the bar colours.
        if (s.idle === 'hide' || s.idleOpacity === 0) return;
        ctx.globalAlpha = s.idleOpacity / 100;
        const mid = mirror || s.idle === 'line' ? y0 + H / 2 : y0 + H - 1.5;
        if (s.idle === 'line') {
          for (let i = 0; i < n; i++) { ctx.fillStyle = fillOf(i); ctx.fillRect(xAt(i), mid - 0.75, bw + gap, 1.5); }
        } else {
          for (let i = 0; i < n; i++) {
            ctx.globalAlpha = (s.idleOpacity / 100) * (0.55 + 0.45 * Math.sin(now / 380 - i * 0.45));
            ctx.fillStyle = fillOf(i);
            ctx.beginPath(); ctx.arc(xAt(i) + bw / 2, mid, Math.max(0.75, Math.min(1.6, bw / 2)), 0, Math.PI * 2); ctx.fill();
          }
        }
        ctx.globalAlpha = 1;
        return;
      }

      const alpha = s.barOpacity / 100;
      if (s.style === 'glow') {
        // The tall soft haze above each spike (reference: neon spectrum).
        ctx.save(); ctx.filter = 'blur(4px)'; ctx.globalAlpha = 0.18 * alpha; bars(st.cur, fillOf, 3); ctx.restore();
      }
      if (s.glow) {
        const gc = s.glowColor === 'custom' ? hex(s.glowCustom) : null;
        ctx.save(); ctx.filter = `blur(${s.glowSize}px)`; ctx.globalAlpha = (s.glowStrength / 100) * alpha;
        draw(gc ? () => gc : fillOf);
        ctx.restore();
      }
      ctx.globalAlpha = alpha;
      draw(fillOf);
      ctx.globalAlpha = 1;
    };
    raf = requestAnimationFrame(frame);
    return () => cancelAnimationFrame(raf);
  });
</script>

<div class="pill eq" class:vertical style:--len="{s.length}px" aria-label="Audio equalizer">
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  .eq { padding: 4px 8px; }
  canvas { display: block; width: var(--len); height: 100%; }
  .vertical canvas { width: 100%; height: var(--len); }
</style>
