<script lang="ts">
  // Fullscreen overlay over a frozen screenshot. OCR and Region: drag a box. Window and
  // Scrolling: hover highlights a window, click picks it. Esc or right-click cancels.
  import { listen } from '@tauri-apps/api/event';
  import { tick } from 'svelte';
  import { ipc, type SnapView, type WinRect } from '../lib/ipc';
  import { profile, store, updateWidgetSettings } from '../lib/config.svelte';
  import { formatColor, pushColor, textOn, toHex } from '../widgets/ocr/color';

  let view = $state<SnapView | null>(null);
  let start = $state<{ x: number; y: number } | null>(null);
  let cur = $state<{ x: number; y: number } | null>(null);
  let hover = $state<WinRect | null>(null);
  let img = $state<HTMLImageElement>();
  const ocrW = $derived(store.config ? profile().widgets.find((w) => w.id === 'ocr') : undefined);
  const ocr = $derived(ocrW?.settings ?? {});
  const picking = $derived(view?.purpose === 'window' || view?.purpose === 'scroll');
  const k = () => (img && innerWidth ? img.naturalWidth / innerWidth : devicePixelRatio); // screenshot px per CSS px
  const hint = $derived(!view ? '' : view.purpose === 'display' ? 'Click a display to capture it (or press its number)'
    : view.purpose === 'color' ? 'Click to copy a colour · arrows to nudge · wheel to zoom'
    : view.purpose === 'ocr'
    ? `Drag to grab text · ${ocr.mode ?? 'text'}`
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
    if (v.purpose === 'color') readPixels(pic);
    await tick();
    setTimeout(() => ipc.ocrSnapShow(), 16); // (no rAF: a hidden window may not run it)
  }
  $effect(() => {
    load();
    const off = listen('snap:ready', load);
    return () => { off.then((f) => f()); };
  });

  // The hint sits at the top of the display with the pointer (the overlay may span several).
  const hintAt = $derived.by(() => {
    if (!view?.displays?.length) return null;
    const c = view.cursor;
    const d = (c && view.displays.find(([x, y, w, h]) => c[0] >= x && c[0] < x + w && c[1] >= y && c[1] < y + h)) ?? view.displays[0];
    const s = k();
    return { x: (d[0] + d[2] / 2) / s, y: d[1] / s + 18 };
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

  // Leaving keeps the screenshot on screen until the window has hidden: clearing it first
  // showed a black window for a moment (the window behind the picture is black).
  function cancel() { start = cur = null; hover = null; ipc.ocrSnapCancel(); }
  async function finishWindow(w: WinRect) {
    hover = null;
    await ipc.ocrSnapFinish(w.x, w.y, w.w, w.h, w.id).catch(() => {});
  }
  async function finishRect() {
    const r = rect;
    start = cur = null;
    if (!r || r.w < 4 || r.h < 4) return;
    const s = k();
    await ipc.ocrSnapFinish(Math.round(r.x * s), Math.round(r.y * s), Math.round(r.w * s), Math.round(r.h * s), null).catch(() => {});
  }
  function down(e: PointerEvent) {
    if (e.button === 2) return cancel();
    if (pickingDisplay) { const i = displayAt(e.clientX, e.clientY); if (i != null) finishDisplay(i); return; }
    if (color) { move(e); pickColor(); return; }
    if (picking) { const w = windowAt(e.clientX, e.clientY); if (w) finishWindow(w); return; }
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    start = cur = { x: e.clientX, y: e.clientY };
  }
  // ---- Full screen with several displays: pick one (hover highlights it) or all of them ----
  const pickingDisplay = $derived(view?.purpose === 'display');
  let hoverDisplay = $state<number | null>(null);
  const displayAt = (x: number, y: number) => {
    const s = k();
    const i = view?.displays.findIndex(([dx, dy, dw, dh]) => x * s >= dx && x * s < dx + dw && y * s >= dy && y * s < dy + dh) ?? -1;
    return i >= 0 ? i : null;
  };
  async function finishDisplay(i: number | 'all') {
    if (!view || !img) return;
    const [x, y, w, h] = i === 'all' ? [0, 0, img.naturalWidth, img.naturalHeight] : view.displays[i];
    hoverDisplay = null;
    await ipc.ocrSnapFinish(x, y, w, h, null).catch(() => {});
  }
  const displayBox = $derived(pickingDisplay && hoverDisplay != null && view ? (() => { const s = k(); const [x, y, w, h] = view.displays[hoverDisplay]; return { x: x / s, y: y / s, w: w / s, h: h / s }; })() : null);

  // ---- Colour picker: a loupe over the frozen screen ----
  const color = $derived(view?.purpose === 'color');
  let px: ImageData | null = null;
  /** The pixel under the loupe, in screenshot pixels. */
  let pos = $state<{ x: number; y: number } | null>(null);
  let zoom = $state(10);
  const LOUPE = 132;
  let loupe = $state<HTMLCanvasElement>();
  function readPixels(pic: HTMLImageElement) {
    const c = document.createElement('canvas');
    c.width = pic.naturalWidth; c.height = pic.naturalHeight;
    const g = c.getContext('2d', { willReadFrequently: true })!;
    g.drawImage(pic, 0, 0);
    px = g.getImageData(0, 0, c.width, c.height);
    pos = view?.cursor ? { x: view.cursor[0], y: view.cursor[1] } : null; // the loupe starts at the pointer
  }
  const hexAt = (p: { x: number; y: number }) => {
    if (!px) return '#000000';
    const i = (Math.min(px.height - 1, Math.max(0, p.y)) * px.width + Math.min(px.width - 1, Math.max(0, p.x))) * 4;
    return toHex(px.data[i], px.data[i + 1], px.data[i + 2]);
  };
  const hex = $derived(pos ? hexAt(pos) : null);
  // Draw the loupe: an odd number of pixels across, each `zoom` CSS px, the centre one boxed.
  $effect(() => {
    if (!loupe || !pos || !px) return;
    const n = Math.floor(LOUPE / zoom) | 1, half = n >> 1;
    const g = loupe.getContext('2d')!;
    const dpr = devicePixelRatio || 1;
    loupe.width = LOUPE * dpr; loupe.height = LOUPE * dpr;
    g.scale(dpr, dpr);
    const off = (LOUPE - n * zoom) / 2;
    for (let dy = -half; dy <= half; dy++) for (let dx = -half; dx <= half; dx++) {
      g.fillStyle = hexAt({ x: pos.x + dx, y: pos.y + dy });
      g.fillRect(off + (dx + half) * zoom, off + (dy + half) * zoom, zoom, zoom);
    }
    g.strokeStyle = 'rgba(0,0,0,0.12)'; g.lineWidth = 0.5;
    for (let i = 0; i <= n; i++) { const a = off + i * zoom; g.beginPath(); g.moveTo(a, 0); g.lineTo(a, LOUPE); g.moveTo(0, a); g.lineTo(LOUPE, a); g.stroke(); }
    const c = off + half * zoom;
    g.lineWidth = 2; g.strokeStyle = '#fff'; g.strokeRect(c, c, zoom, zoom);
    g.lineWidth = 1; g.strokeStyle = '#000'; g.strokeRect(c - 1, c - 1, zoom + 2, zoom + 2);
  });
  // Where the loupe sits: beside the pointer, flipped near the screen's edges.
  const at = $derived.by(() => {
    if (!pos) return null;
    const s = k(), cx = pos.x / s, cy = pos.y / s, gap = 24, h = LOUPE + 34;
    return { x: cx + gap + LOUPE > innerWidth ? cx - gap - LOUPE : cx + gap, y: cy + gap + h > innerHeight ? cy - gap - h : cy + gap };
  });
  async function pickColor() {
    if (!pos) return;
    const h = hexAt(pos);
    const text = formatColor(h, (ocr.colorFormat as 'hex' | 'rgb' | 'hsl') ?? 'hex');
    if (ocrW) updateWidgetSettings(ocrW.instance, { colors: pushColor((ocr.colors as string[]) ?? [], h) });
    pos = null;
    await ipc.colorPickFinish(h, text).catch(() => {});
  }
  function colorKey(e: KeyboardEvent) {
    if (!color || !pos || !px) return;
    const step = e.shiftKey ? 10 : 1;
    const d = { ArrowLeft: [-step, 0], ArrowRight: [step, 0], ArrowUp: [0, -step], ArrowDown: [0, step] }[e.key];
    if (d) { e.preventDefault(); pos = { x: Math.min(px.width - 1, Math.max(0, pos.x + d[0])), y: Math.min(px.height - 1, Math.max(0, pos.y + d[1])) }; }
    else if (e.key === 'Enter') { e.preventDefault(); pickColor(); }
  }
  function wheel(e: WheelEvent) { if (color) zoom = Math.min(20, Math.max(6, zoom + (e.deltaY < 0 ? 2 : -2))); }

  function move(e: PointerEvent) {
    if (pickingDisplay) { hoverDisplay = displayAt(e.clientX, e.clientY); return; }
    if (color) { const s = k(); pos = { x: Math.floor(e.clientX * s), y: Math.floor(e.clientY * s) }; return; }
    if (picking) hover = windowAt(e.clientX, e.clientY);
    else if (start) cur = { x: e.clientX, y: e.clientY };
  }
</script>

<svelte:window onkeydown={(e) => {
  if (e.key === 'Escape') return cancel();
  if (pickingDisplay && view) {
    if (e.key.toLowerCase() === 'a') return finishDisplay('all');
    const n = parseInt(e.key, 10);
    if (n >= 1 && n <= view.displays.length) return finishDisplay(n - 1);
    return;
  }
  colorKey(e);
}} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="snap" class:picking={picking || pickingDisplay} class:color style:background-image={view ? `url(${view.img})` : 'none'}
     onpointerdown={down} onpointermove={move} onwheel={wheel} onpointerup={() => { if (!picking) finishRect(); }} oncontextmenu={(e) => e.preventDefault()}>
  {#if view}<img bind:this={img} src={view.img} alt="" class="probe" />{/if}
  {#if box && view}
    <div class="sel" style:left="{box.x}px" style:top="{box.y}px" style:width="{box.w}px" style:height="{box.h}px"
         style:background-image="url({view.img})" style:background-position="{-box.x}px {-box.y}px">
      {#if picking && hover}<span class="tag">{hover.title}</span>{/if}
    </div>
  {/if}
  {#if color && pos && at && hex}
    <div class="loupe" style:left="{at.x}px" style:top="{at.y}px">
      <canvas bind:this={loupe} style:width="{LOUPE}px" style:height="{LOUPE}px"></canvas>
      <div class="val"><i style:background={hex} style:color={textOn(hex)}></i><b>{hex}</b><span>{formatColor(hex, 'rgb').slice(4, -1)}</span></div>
    </div>
  {/if}
  {#if displayBox && view && hoverDisplay != null}
    <div class="sel disp" style:left="{displayBox.x}px" style:top="{displayBox.y}px" style:width="{displayBox.w}px" style:height="{displayBox.h}px"
         style:background-image="url({view.img})" style:background-position="{-displayBox.x}px {-displayBox.y}px">
      <span class="tag big">{view.displayNames[hoverDisplay] ?? `Display ${hoverDisplay + 1}`}</span>
    </div>
  {/if}
  {#if pickingDisplay && view && hintAt}
    <!-- Above the hint: one button per display, and All displays. -->
    <div class="dpick" style:left="{hintAt.x}px" style:top="{hintAt.y + 46}px" role="group" aria-label="Choose a display">
      {#each view.displays as _, i (i)}
        <button onpointerdown={(e) => e.stopPropagation()} onclick={() => finishDisplay(i)} onpointerenter={() => (hoverDisplay = i)}>
          <b>{i + 1}</b>{(view.displayNames[i] ?? '').replace(/^Display \d+ · /, '')}
        </button>
      {/each}
      <button class="all" onpointerdown={(e) => e.stopPropagation()} onclick={() => finishDisplay('all')}><b>A</b>All displays</button>
    </div>
  {/if}
  {#if view}<div class="hint" style:left={hintAt ? `${hintAt.x}px` : null} style:top={hintAt ? `${hintAt.y}px` : null}>{hint} · Esc to cancel</div>{/if}
</div>

<style>
  /* Scoped to the snap window: every window shares one stylesheet, and an unscoped rule
     painted the dock and panels black behind their glass. */
  :global(html[data-window='snap']), :global(html[data-window='snap'] body) { margin: 0; background: #000; overflow: hidden; }
  .snap { position: fixed; inset: 0; cursor: crosshair; background-size: 100% 100%; user-select: none; }
  .snap.picking { cursor: pointer; }
  /* Picking a display: the highlighted one shows bright with its name in the middle. */
  .sel.disp { outline-width: 3px; display: grid; place-items: center; }
  .tag.big { position: static; max-width: none; padding: 8px 16px; font-size: 15px; border-radius: 12px; }
  .dpick { position: absolute; transform: translateX(-50%); display: flex; gap: 6px; padding: 6px; border-radius: 14px;
           background: rgba(20, 20, 24, 0.8); backdrop-filter: blur(var(--blur)); z-index: 2; }
  .dpick button { display: flex; align-items: center; gap: 8px; height: 34px; padding: 0 12px; border-radius: 10px; color: #fff;
                  font: 500 12.5px Poppins, 'Segoe UI Variable', sans-serif; background: rgba(255, 255, 255, 0.08); cursor: pointer; white-space: nowrap; }
  .dpick button:hover { background: #0a84ff; }
  .dpick b { display: grid; place-items: center; width: 20px; height: 20px; border-radius: 6px; background: rgba(255, 255, 255, 0.18); font-size: 11px; }
  .dpick .all { background: rgba(10, 132, 255, 0.35); }
  /* Colour picking: the screen as it is (no dimming), a fine crosshair, the loupe beside it. */
  .snap.color::before { display: none; }
  .loupe { position: absolute; pointer-events: none; display: flex; flex-direction: column; align-items: center; gap: 6px; }
  .loupe canvas { border-radius: 50%; box-shadow: 0 0 0 3px #fff, 0 0 0 4px rgba(0,0,0,0.35), 0 10px 30px rgba(0,0,0,0.45); background: #000; }
  .val { display: flex; align-items: center; gap: 7px; padding: 5px 10px 5px 6px; border-radius: 999px; background: rgba(20, 20, 24, 0.85); color: #fff;
         font: 500 12px Poppins, 'Segoe UI Variable', sans-serif; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .val i { width: 16px; height: 16px; border-radius: 50%; box-shadow: inset 0 0 0 1px rgba(255,255,255,0.4); }
  .val span { opacity: 0.7; }
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
