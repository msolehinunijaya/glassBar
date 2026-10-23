<script lang="ts">
  // Fullscreen overlay over a frozen screenshot: drag a box, release to OCR it, Esc cancels.
  import { listen } from '@tauri-apps/api/event';
  import { ipc } from '../lib/ipc';
  import { profile, store } from '../lib/config.svelte';

  let src = $state('');
  let start = $state<{ x: number; y: number } | null>(null);
  let cur = $state<{ x: number; y: number } | null>(null);
  let img: HTMLImageElement;
  const mode = $derived((store.config && profile().widgets.find((w) => w.id === 'ocr')?.settings?.mode) || 'text');
  const engine = $derived((store.config && profile().widgets.find((w) => w.id === 'ocr')?.settings?.engine) === 'ai' ? 'Claude' : 'Windows OCR');

  async function load() {
    start = cur = null;
    src = (await ipc.ocrShot()) ?? '';
  }
  $effect(() => {
    load();
    const off = listen('snap:ready', load);
    return () => { off.then((f) => f()); };
  });

  const rect = $derived(start && cur ? {
    x: Math.min(start.x, cur.x), y: Math.min(start.y, cur.y),
    w: Math.abs(cur.x - start.x), h: Math.abs(cur.y - start.y),
  } : null);

  function cancel() { src = ''; start = cur = null; ipc.ocrSnapCancel(); }
  async function finish() {
    const r = rect;
    start = cur = null;
    if (!r || r.w < 4 || r.h < 4 || !img) return;
    const k = img.naturalWidth / innerWidth; // CSS px -> screenshot px
    src = '';
    await ipc.ocrSnapFinish(Math.round(r.x * k), Math.round(r.y * k), Math.round(r.w * k), Math.round(r.h * k)).catch(() => {});
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && cancel()} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="snap" style:background-image={src ? `url(${src})` : 'none'}
     onpointerdown={(e) => { if (e.button === 2) return cancel(); (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId); start = cur = { x: e.clientX, y: e.clientY }; }}
     onpointermove={(e) => { if (start) cur = { x: e.clientX, y: e.clientY }; }}
     onpointerup={finish}>
  {#if src}<img bind:this={img} {src} alt="" class="probe" />{/if}
  {#if rect}
    <div class="sel" style:left="{rect.x}px" style:top="{rect.y}px" style:width="{rect.w}px" style:height="{rect.h}px"
         style:background-image="url({src})" style:background-position="{-rect.x}px {-rect.y}px"></div>
  {/if}
  <div class="hint">Drag to grab text · {mode} · {engine} · Esc to cancel</div>
</div>

<style>
  :global(html), :global(body) { margin: 0; background: #000; overflow: hidden; }
  .snap { position: fixed; inset: 0; cursor: crosshair; background-size: 100% 100%; user-select: none; }
  .snap::before { content: ''; position: absolute; inset: 0; background: rgba(0, 0, 0, 0.35); pointer-events: none; }
  .probe { display: none; }
  .sel { position: absolute; background-size: 100vw 100vh; outline: 2px solid #0a84ff; box-shadow: 0 0 0 1px rgba(255,255,255,0.6); }
  .hint {
    position: absolute; top: 18px; left: 50%; transform: translateX(-50%); padding: 8px 16px; border-radius: 12px;
    background: rgba(20, 20, 24, 0.75); color: #fff; font: 500 13px Poppins, 'Segoe UI Variable', sans-serif;
    backdrop-filter: blur(12px); pointer-events: none; text-transform: none;
  }
</style>
