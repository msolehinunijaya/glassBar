<script lang="ts">
  // The live mirror. The camera runs only while this panel is open: closing it unmounts the
  // panel, which stops every camera track, so the webcam light goes off.
  //
  // Photos and videos save what you see (flip, light, zoom), drawn through a canvas.
  import { Camera, Circle, Ellipsis, TrianglesCenterlineDashedVertical as FlipIcon, FolderOpen, Lightbulb, RectangleHorizontal, RefreshCw, Square, Sun, VideoOff, ZoomIn } from '@lucide/svelte';
  import PanelHeader from '../../lib/PanelHeader.svelte';
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc } from '../../lib/ipc';
  import type { PopupProps } from '../types';
  import { clock, cropOf, filterOf, type MirrorSettings } from './mirror';

  let { instance, settings }: PopupProps<MirrorSettings> = $props();

  // Slider values while dragging; saved to settings when let go.
  let live = $state<Partial<MirrorSettings>>({});
  const v = $derived({ ...settings, ...live });
  const save = (patch: Partial<MirrorSettings>) => { updateWidgetSettings(instance, patch); live = {}; };

  let video = $state<HTMLVideoElement>();
  let stream = $state<MediaStream | null>(null);
  let error = $state('');
  let tries = $state(0);
  let cams = $state<{ id: string; name: string }[]>([]);
  let more = $state(false);

  const stopAll = (s: MediaStream) => s.getTracks().forEach((t) => t.stop());

  function messageOf(e: unknown) {
    const n = (e as DOMException)?.name;
    if (n === 'NotAllowedError') return 'Camera access is blocked. Turn on Windows Settings › Privacy & security › Camera › "Let desktop apps access your camera".';
    if (n === 'NotFoundError' || n === 'OverconstrainedError') return 'No camera found. Plug one in and try again.';
    if (n === 'NotReadableError') return 'The camera is busy in another app. Close it there and try again.';
    return 'The camera could not start.';
  }

  // Every settings change hands us a new settings object: read the camera through a derived
  // value, so only picking another camera (or Try again) restarts it, not a slider.
  const camId = $derived(settings.camera);
  $effect(() => {
    const id = camId;
    void tries;
    let gone = false;
    let s: MediaStream | null = null;
    error = '';
    const size = { width: { ideal: 1280 }, height: { ideal: 720 } };
    navigator.mediaDevices.getUserMedia({ video: id ? { ...size, deviceId: { exact: id } } : size, audio: false })
      .then(async (st) => {
        if (gone) { stopAll(st); return; }
        s = stream = st;
        const all = await navigator.mediaDevices.enumerateDevices();
        cams = all.filter((d) => d.kind === 'videoinput').map((d, i) => ({ id: d.deviceId, name: d.label || `Camera ${i + 1}` }));
      })
      .catch((e) => {
        if (gone) return;
        // The chosen camera was unplugged: fall back to the default one.
        if (id && (e?.name === 'NotFoundError' || e?.name === 'OverconstrainedError')) { save({ camera: '' }); return; }
        error = messageOf(e);
      });
    return () => { gone = true; stopRec(); if (s) stopAll(s); stream = null; };
  });

  $effect(() => { if (video) video.srcObject = stream; });

  /** Draw what the preview shows onto `ctx` (size of the crop). */
  function draw(ctx: CanvasRenderingContext2D, r: { x: number; y: number; w: number; h: number }) {
    ctx.save();
    ctx.filter = filterOf(v);
    if (v.flip) { ctx.translate(r.w, 0); ctx.scale(-1, 1); }
    ctx.drawImage(video!, r.x, r.y, r.w, r.h, 0, 0, r.w, r.h);
    ctx.restore();
  }
  const cropNow = () => cropOf(video!.videoWidth, video!.videoHeight, v.zoom, v.shape);

  // ── Saved toast ──
  let saved = $state<{ path: string; what: string } | null>(null);
  let savedTimer: ReturnType<typeof setTimeout>;
  async function store(kind: 'photo' | 'video', blob: Blob) {
    try {
      const path = await ipc.mirrorSave(kind, new Uint8Array(await blob.arrayBuffer()));
      saved = { path, what: kind === 'photo' ? 'Photo saved' : 'Video saved' };
      clearTimeout(savedTimer);
      savedTimer = setTimeout(() => (saved = null), 6000);
    } catch (e) {
      saved = { path: '', what: `Couldn't save: ${e}` };
    }
  }
  $effect(() => () => clearTimeout(savedTimer));

  // ── Photo ──
  let count = $state(0);
  let flash = $state(0);
  async function shoot() {
    if (!stream || !video?.videoWidth || count) return;
    if (v.countdown) {
      for (count = 3; count > 0; count--) await new Promise((r) => setTimeout(r, 1000));
      if (!stream) return;
    }
    const r = cropNow();
    const c = document.createElement('canvas');
    c.width = Math.round(r.w); c.height = Math.round(r.h);
    const ctx = c.getContext('2d')!;
    if (v.shape === 'circle') { ctx.beginPath(); ctx.arc(c.width / 2, c.height / 2, c.width / 2, 0, Math.PI * 2); ctx.clip(); }
    draw(ctx, r);
    flash++;
    c.toBlob((b) => b && store('photo', b), 'image/png');
  }

  // ── Video ──
  let rec: MediaRecorder | null = null;
  let recFrame = 0;
  let recStart = $state(0);
  let recSecs = $state(0);
  let recTick: ReturnType<typeof setInterval>;
  function startRec() {
    if (!stream || !video?.videoWidth) return;
    const r = cropNow();
    const c = document.createElement('canvas');
    c.width = Math.round(r.w) & ~1; c.height = Math.round(r.h) & ~1; // encoders want even sizes
    const ctx = c.getContext('2d')!;
    const paint = () => { draw(ctx, cropNow()); recFrame = requestAnimationFrame(paint); };
    paint();
    const chunks: Blob[] = [];
    const type = ['video/webm;codecs=vp9', 'video/webm'].find((t) => MediaRecorder.isTypeSupported(t));
    rec = new MediaRecorder(c.captureStream(30), type ? { mimeType: type } : undefined);
    rec.ondataavailable = (e) => { if (e.data.size) chunks.push(e.data); };
    // Also runs when the panel closes mid-recording: what was recorded is still saved.
    rec.onstop = () => { if (chunks.length) store('video', new Blob(chunks, { type: 'video/webm' })); };
    rec.start(1000);
    recStart = Date.now(); recSecs = 0;
    recTick = setInterval(() => (recSecs = (Date.now() - recStart) / 1000), 250);
  }
  function stopRec() {
    cancelAnimationFrame(recFrame);
    clearInterval(recTick);
    if (rec?.state === 'recording') rec.stop();
    rec = null; recStart = 0;
  }

  const pct = (x: number, lo: number, hi: number) => `${((x - lo) / (hi - lo)) * 100}%`;
  const ring = $derived(v.ring / 100);
</script>

<PanelHeader title="Mirror">
  <button class="icon-btn" class:on={v.shape === 'circle'} title="Circle" aria-label="Circle" aria-pressed={v.shape === 'circle'} onclick={() => save({ shape: 'circle' })}><Circle size={16} /></button>
  <button class="icon-btn" class:on={v.shape === 'square'} title="Square" aria-label="Square" aria-pressed={v.shape === 'square'} onclick={() => save({ shape: 'square' })}><Square size={16} /></button>
  <button class="icon-btn" class:on={v.shape === 'rect'} title="Rectangle" aria-label="Rectangle" aria-pressed={v.shape === 'rect'} onclick={() => save({ shape: 'rect' })}><RectangleHorizontal size={16} /></button>
  <button class="icon-btn" class:on={more} title="More options" aria-label="More options" aria-expanded={more} onclick={() => (more = !more)}><Ellipsis size={16} /></button>
</PanelHeader>

<div class="stage">
  <div class="view" class:circle={v.shape === 'circle'} class:square={v.shape === 'square'}
       style:aspect-ratio={v.shape === 'circle' || v.shape === 'square' ? '1' : video?.videoWidth ? `${video.videoWidth} / ${video.videoHeight}` : '16 / 9'}
       style:box-shadow={ring ? `0 0 0 ${2 + ring * 10}px rgba(255,255,255,${0.55 + ring * 0.45}), 0 0 ${ring * 40}px ${ring * 14}px rgba(255,255,255,${ring * 0.9})` : 'none'}>
    {#if error}
      <div class="msg">
        <VideoOff size={28} />
        <p>{error}</p>
        <button class="btn" onclick={() => tries++}><RefreshCw size={14} /> Try again</button>
      </div>
    {:else}
      <!-- svelte-ignore a11y_media_has_caption -->
      <video bind:this={video} autoplay muted playsinline
             style:filter={filterOf(v)} style:transform="scale({v.flip ? -v.zoom : v.zoom}, {v.zoom})"></video>
      {#if !stream}<div class="msg dim">Starting the camera…</div>{/if}
      {#if v.grid}<div class="grid" aria-hidden="true"></div>{/if}
      {#if count}<div class="count" aria-live="assertive">{count}</div>{/if}
      {#key flash}{#if flash}<div class="flash" aria-hidden="true"></div>{/if}{/key}
      {#if recStart}<div class="rec" aria-live="polite"><i></i>{clock(recSecs)}</div>{/if}
    {/if}
  </div>
</div>

<div class="bar">
  <button class="icon-btn" class:on={v.flip} title={v.flip ? 'Flipped like a mirror' : 'Not flipped'} aria-label="Flip" aria-pressed={v.flip} onclick={() => save({ flip: !v.flip })}><FlipIcon size={18} /></button>
  <button class="shoot" title="Take a photo" aria-label="Take a photo" disabled={!stream || !!count} onclick={shoot}><Camera size={20} /></button>
  <button class="icon-btn recbtn" class:going={!!recStart} title={recStart ? 'Stop recording' : 'Record a video'} aria-label={recStart ? 'Stop recording' : 'Record a video'}
          disabled={!stream} onclick={() => (recStart ? stopRec() : startRec())}>
    {#if recStart}<Square size={14} fill="currentColor" />{:else}<i class="dot"></i>{/if}
  </button>
</div>

<div class="sliders">
  <label title="Brightness"><Sun size={15} class="dim" />
    <input type="range" class="slider" min="50" max="200" value={v.brightness} style:--fill={pct(v.brightness, 50, 200)} aria-label="Brightness"
           oninput={(e) => (live = { ...live, brightness: +e.currentTarget.value })} onchange={(e) => save({ brightness: +e.currentTarget.value })} /></label>
  <label title="Ring light"><Lightbulb size={15} class="dim" />
    <input type="range" class="slider" min="0" max="100" value={v.ring} style:--fill={pct(v.ring, 0, 100)} aria-label="Ring light"
           oninput={(e) => (live = { ...live, ring: +e.currentTarget.value })} onchange={(e) => save({ ring: +e.currentTarget.value })} /></label>
  <label title="Zoom"><ZoomIn size={15} class="dim" />
    <input type="range" class="slider" min="1" max="3" step="0.1" value={v.zoom} style:--fill={pct(v.zoom, 1, 3)} aria-label="Zoom"
           oninput={(e) => (live = { ...live, zoom: +e.currentTarget.value })} onchange={(e) => save({ zoom: +e.currentTarget.value })} /></label>
</div>

{#if more}
  <div class="more">
    <OptionsPopover title="More" values={v} save={(p) => save(p as Partial<MirrorSettings>)} options={[
      { type: 'range', key: 'contrast', label: 'Contrast', min: 50, max: 200, unit: '%' },
      { type: 'toggle', key: 'grid', label: 'Grid' },
      { type: 'toggle', key: 'countdown', label: '3-second timer', hint: 'Before a photo' },
      ...(cams.length > 1 ? [{ type: 'choice' as const, key: 'camera', label: 'Camera', menu: true,
        choices: [{ value: '', label: 'Default' }, ...cams.map((c) => ({ value: c.id, label: c.name }))] }] : []),
    ]} />
  </div>
{/if}

{#if saved}
  <div class="toast" role="status">
    <span>{saved.what}</span>
    {#if saved.path}<button class="btn" onclick={() => ipc.downloadsReveal(saved!.path).catch(() => {})}><FolderOpen size={14} /> Show in folder</button>{/if}
  </div>
{/if}

<style>
  .stage { display: grid; place-items: center; padding: var(--s-3) 0; }
  .view {
    position: relative; width: 100%; overflow: hidden; background: #000;
    border-radius: var(--r-outer); transition: border-radius 0.3s var(--spring), width 0.3s var(--spring), box-shadow 0.2s var(--ease);
  }
  .view.circle { width: 82%; border-radius: 50%; }
  /* Square: the same size as the circle, with the panel's inner corners. */
  .view.square { width: 82%; }
  video { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; transition: transform 0.2s var(--ease); }
  .msg { position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: var(--s-2);
         padding: var(--s-4); text-align: center; font-size: 12px; color: #fff; }
  .msg p { margin: 0; line-height: 1.4; }
  .grid { position: absolute; inset: 0; pointer-events: none;
          background: linear-gradient(to right, transparent calc(33.33% - 0.5px), rgba(255,255,255,0.45) 0 calc(33.33% + 0.5px), transparent 0 calc(66.66% - 0.5px), rgba(255,255,255,0.45) 0 calc(66.66% + 0.5px), transparent 0),
                      linear-gradient(to bottom, transparent calc(33.33% - 0.5px), rgba(255,255,255,0.45) 0 calc(33.33% + 0.5px), transparent 0 calc(66.66% - 0.5px), rgba(255,255,255,0.45) 0 calc(66.66% + 0.5px), transparent 0); }
  .count { position: absolute; inset: 0; display: grid; place-items: center; font-size: 64px; font-weight: 600; color: #fff;
           text-shadow: 0 2px 12px rgba(0,0,0,0.5); font-variant-numeric: tabular-nums; }
  .flash { position: absolute; inset: 0; background: #fff; pointer-events: none; animation: flash 0.35s var(--ease) forwards; }
  @keyframes flash { from { opacity: 0.9; } to { opacity: 0; } }
  .rec { position: absolute; top: 10px; left: 50%; translate: -50% 0; display: flex; align-items: center; gap: 6px;
         padding: 3px 8px; border-radius: var(--r-chip); background: rgba(0,0,0,0.55); color: #fff;
         font-size: 11px; font-weight: 600; font-variant-numeric: tabular-nums; }
  .rec i { width: 7px; height: 7px; border-radius: 50%; background: #ff453a; animation: blink 1s steps(2) infinite; }
  @keyframes blink { 50% { opacity: 0.2; } }

  .icon-btn.on { color: var(--accent); background: var(--bg-2, var(--pill)); }
  .icon-btn:focus-visible, .shoot:focus-visible, .btn:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .icon-btn:disabled, .shoot:disabled { opacity: 0.4; pointer-events: none; }

  .bar { display: flex; align-items: center; justify-content: center; gap: var(--s-6, 24px); margin-bottom: var(--s-3); }
  .shoot { display: grid; place-items: center; width: 48px; height: 48px; border-radius: 50%;
           background: var(--accent); color: var(--on-accent, #fff); transition: transform 0.3s var(--spring), filter 0.2s var(--ease); }
  .shoot:hover { filter: brightness(1.1); }
  .shoot:active { transform: scale(0.9); }
  .dot { display: block; width: 14px; height: 14px; border-radius: 50%; background: #ff453a; }
  .recbtn.going { color: #ff453a; }

  .sliders { display: flex; flex-direction: column; gap: var(--s-2); margin-bottom: var(--s-2); }
  .sliders label { display: grid; grid-template-columns: 20px 1fr; align-items: center; gap: var(--s-2); }
  .more { margin-top: var(--s-2); }

  .btn { display: inline-flex; align-items: center; gap: 6px; padding: 6px 10px; border-radius: var(--r-inner);
         background: var(--bg-2, var(--pill)); font-size: 12px; transition: background 0.2s var(--ease); }
  .btn:hover { background: var(--pill-hover); }
  .msg .btn { background: rgba(255,255,255,0.15); color: #fff; }
  .toast { display: flex; align-items: center; justify-content: space-between; gap: var(--s-2); margin-top: var(--s-2);
           padding: 8px 8px 8px 12px; border-radius: var(--r-inner); background: var(--bg-2, var(--pill)); font-size: 12px; }
</style>
