<script lang="ts">
  // Fullscreen overlay over a frozen screenshot. OCR and Region: drag a box. Window and
  // Scrolling: hover highlights a window, click picks it. Esc or right-click cancels.
  import { listen } from '@tauri-apps/api/event';
  import { tick } from 'svelte';
  import { ipc, type SnapView, type WinRect } from '../lib/ipc';
  import { profile, store } from '../lib/config.svelte';

  let view = $state<SnapView | null>(null);
  let start = $state<{ x: number; y: number } | null>(null);
  let cur = $state<{ x: number; y: number } | null>(null);
  let hover = $state<WinRect | null>(null);
  let img = $state<HTMLImageElement>();
  const ocr = $derived(store.config ? profile().widgets.find((w) => w.id === 'ocr')?.settings ?? {} : {});
  const picking = $derived(view?.purpose === 'window' || view?.purpose === 'scroll');
  const k = () => (img && innerWidth ? img.naturalWidth / innerWidth : devicePixelRatio); // screenshot px per CSS px
  const hint = $derived(!view ? '' : view.purpose === 'ocr'
    ? `Drag to grab text · ${ocr.mode ?? 'text'} · ${ocr.engine === 'ai' ? 'Claude' : 'Windows OCR'}`
    : view.purpose === 'region' ? 'Drag to select an area'
    : view.purpose === 'window' ? 'Click a window to capture it'
    : 'Click a window to capture its whole scrolling page (it will scroll by itself)');

  async function load() {
    start = cur = null; hover = null;
    const v = await ipc.ocrShot();
    if (!v) return;
    // Decode the screenshot before the window appears, so it never flashes black.
    const pic = new Image();
    pic.src = v.img;
    await pic.decode().catch(() => {});
    view = v;
    await tick();
    setTimeout(() => ipc.ocrSnapShow(), 16); // (no rAF: a hidden window may not run it)
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
  // Window under the pointer, in CSS px (windows come front-most first).
  function windowAt(x: number, y: number) {
    const s = k();
    return view?.windows.find((w) => x * s >= w.x && x * s < w.x + w.w && y * s >= w.y && y * s < w.y + w.h) ?? null;
  }
  const box = $derived(picking && hover ? { x: hover.x / k(), y: hover.y / k(), w: hover.w / k(), h: hover.h / k() } : rect);

  function cancel() { view = null; start = cur = null; hover = null; ipc.ocrSnapCancel(); }
  async function finishWindow(w: WinRect) {
    view = null; hover = null;
    await ipc.ocrSnapFinish(w.x, w.y, w.w, w.h, w.id).catch(() => {});
  }
  async function finishRect() {
    const r = rect;
    start = cur = null;
    if (!r || r.w < 4 || r.h < 4) return;
    const s = k();
    view = null;
    await ipc.ocrSnapFinish(Math.round(r.x * s), Math.round(r.y * s), Math.round(r.w * s), Math.round(r.h * s), null).catch(() => {});
  }
  function down(e: PointerEvent) {
    if (e.button === 2) return cancel();
    if (picking) { const w = windowAt(e.clientX, e.clientY); if (w) finishWindow(w); return; }
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    start = cur = { x: e.clientX, y: e.clientY };
  }
  function move(e: PointerEvent) {
    if (picking) hover = windowAt(e.clientX, e.clientY);
    else if (start) cur = { x: e.clientX, y: e.clientY };
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && cancel()} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="snap" class:picking style:background-image={view ? `url(${view.img})` : 'none'}
     onpointerdown={down} onpointermove={move} onpointerup={() => { if (!picking) finishRect(); }} oncontextmenu={(e) => e.preventDefault()}>
  {#if view}<img bind:this={img} src={view.img} alt="" class="probe" />{/if}
  {#if box && view}
    <div class="sel" style:left="{box.x}px" style:top="{box.y}px" style:width="{box.w}px" style:height="{box.h}px"
         style:background-image="url({view.img})" style:background-position="{-box.x}px {-box.y}px">
      {#if picking && hover}<span class="tag">{hover.title}</span>{/if}
    </div>
  {/if}
  {#if view}<div class="hint">{hint} · Esc to cancel</div>{/if}
</div>

<style>
  /* Scoped to the snap window: every window shares one stylesheet, and an unscoped rule
     painted the dock and panels black behind their glass. */
  :global(html[data-window='snap']), :global(html[data-window='snap'] body) { margin: 0; background: #000; overflow: hidden; }
  .snap { position: fixed; inset: 0; cursor: crosshair; background-size: 100% 100%; user-select: none; }
  .snap.picking { cursor: pointer; }
  .snap::before { content: ''; position: absolute; inset: 0; background: rgba(0, 0, 0, 0.35); pointer-events: none; }
  .probe { display: none; }
  .sel { position: absolute; background-size: 100vw 100vh; outline: 2px solid #0a84ff; box-shadow: 0 0 0 1px rgba(255,255,255,0.6); pointer-events: none; }
  .tag { position: absolute; left: 6px; top: 6px; max-width: 60%; padding: 3px 8px; border-radius: calc(8px * var(--rs)); background: #0a84ff; color: #fff;
         font: 500 12px Poppins, 'Segoe UI Variable', sans-serif; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .hint {
    position: absolute; top: 18px; left: 50%; transform: translateX(-50%); padding: 8px 16px; border-radius: calc(12px * var(--rs));
    background: rgba(20, 20, 24, 0.75); color: #fff; font: 500 13px Poppins, 'Segoe UI Variable', sans-serif;
    backdrop-filter: blur(var(--blur)); pointer-events: none;
  }
</style>
