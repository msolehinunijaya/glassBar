<script lang="ts">
  // Live spectrum on a canvas. The backend sends 96 bands 0..255 (~30/s) while this is mounted, the
  // window is visible and (with battery saver) the PC is plugged in. Each frame resamples them
  // to the user's bar count, smooths and draws (draw.ts). Nothing playing for 0.4 s: bars fall
  // to zero, then the idle look.
  import { listen } from '@tauri-apps/api/event';
  import { ipc, type Battery } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { liveMedia } from '../music/media.svelte';
  import type { PillProps } from '../types';
  import { paint } from './draw';
  import { palette, resample, step } from './eq';
  import type { EqSettings } from './settings';

  let { settings: s, vertical }: PillProps<EqSettings> = $props();
  let canvas: HTMLCanvasElement;
  let src: number[] = [];
  let lastData = 0;
  let album: [string, string] | null = null;

  const bat = live<Battery | null>('battery', ipc.batteryGet, null);
  const media = liveMedia();
  const paused = () => s.batterySaver && !!bat.v && !bat.v.onAc;

  $effect(() => {
    const lease = () => { if (document.visibilityState === 'visible' && !paused()) ipc.audioSpectrum(); };
    lease();
    const t = setInterval(lease, 2000);
    const off = listen<number[]>('audio:spectrum', (e) => { src = e.payload.map((v) => v / 255); lastData = performance.now(); });
    return () => { clearInterval(t); off.then((f) => f()); };
  });

  // Album colours: two vivid colours from the cover, re-read when the cover changes.
  $effect(() => {
    const art = s.colorMode === 'album' ? media.m?.art : null;
    if (!art) { album = null; return; }
    const img = new Image();
    img.onload = () => {
      const c = document.createElement('canvas');
      c.width = c.height = 24;
      const x = c.getContext('2d', { willReadFrequently: true })!;
      x.drawImage(img, 0, 0, 24, 24);
      album = palette(x.getImageData(0, 0, 24, 24).data);
    };
    img.src = art;
  });

  $effect(() => {
    const ctx = canvas.getContext('2d')!;
    const st = { cur: [] as number[], peak: [] as number[], hold: [] as number[] };
    let raf = 0, last = 0, quiet = true;
    // Size and accent are read when they change / once a second, not every frame: reading them
    // forces the page to recalculate styles and layout.
    let w = canvas.clientWidth, h = canvas.clientHeight, accent = '', accentAt = -1e9;
    const ro = new ResizeObserver(() => { w = canvas.clientWidth; h = canvas.clientHeight; });
    ro.observe(canvas);

    const frame = (now: number) => {
      raf = requestAnimationFrame(frame);
      if (now - last < (quiet ? 66 : 1000 / +s.fps - 2)) return; // idle redraws at ~15 fps
      const dt = last ? Math.min(100, now - last) : 16;
      last = now;

      const dpr = devicePixelRatio || 1;
      if (!w || !h) return;
      if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
        canvas.width = Math.round(w * dpr); canvas.height = Math.round(h * dpr);
      }
      const n = Math.max(1, Math.round(s.bands));
      const signal = !paused() && now - lastData < 400;
      const target = signal
        ? resample(src, n, +s.lowHz, +s.highHz, s.spacing, s.tilt / 100).map((v) => (v * s.gain) / 100)
        : Array(n).fill(0);
      if (s.flip) target.reverse();
      step(st, target, dt, now, s.fall * 0.5, s.peaks ? s.peakHold : 0);
      quiet = !signal && st.cur.every((v) => v < 0.01) && st.peak.every((v) => v < 0.01);

      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.clearRect(0, 0, w, h);
      if (vertical) { ctx.translate(w, 0); ctx.rotate(Math.PI / 2); } // draw along the length, bars grow right
      if (now - accentAt > 1000) { accent = getComputedStyle(canvas).getPropertyValue('--accent').trim(); accentAt = now; }
      paint(ctx, s, { cur: st.cur, peak: st.peak, quiet, now }, vertical ? h : w, vertical ? w : h, accent, album);
    };
    raf = requestAnimationFrame(frame);
    return () => { cancelAnimationFrame(raf); ro.disconnect(); };
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
