<script lang="ts">
  import { availableMonitors, getCurrentWindow, primaryMonitor } from '@tauri-apps/api/window';
  import { flip } from 'svelte/animate';
  import { onMount } from 'svelte';
  import { Check, Maximize2, Plus, X } from '@lucide/svelte';
  import { profile, store, update } from '../lib/config.svelte';
  import { events, ipc, type WidgetInst, type WidgetSize } from '../lib/ipc';
  import { insertIndex, snap } from '../lib/geometry';
  import { GALLERY, openPopup } from '../lib/popup';
  import { chargerTone, chime, deviceTone, playFile, shutter, tick } from '../lib/sound';
  // Bundled adzan: "Beautiful adhan" by Adam-synagda, Wikimedia Commons, CC0.
  import adhanUrl from '../assets/adhan.ogg?url';
  import { showMenu } from '../lib/menu';
  import { PredefinedMenuItem } from '@tauri-apps/api/menu';
  import { settingsOf, widgets } from '../widgets/registry';
  import { visibleOnDock } from '../widgets/layout';
  import { jumpTarget } from '../lib/notify';
  import { PARTS, partPanel } from '../widgets/productivity/parts';

  const win = getCurrentWindow();
  let edit = $state(false);
  let scroller: HTMLDivElement;

  const bar = $derived(profile().bar);
  const vertical = $derived(bar.edge === 'left' || bar.edge === 'right');
  // Widgets that are on; the standalone Start & Search hides while the App Launcher shows it inline.
  const placed = $derived(visibleOnDock(profile().widgets, widgets));

  // While dragging a widget in edit mode we show a local order, committed on drop.
  let order = $state<string[] | null>(null);
  const shown = $derived(order ? order.map((id) => placed.find((w) => w.instance === id)!).filter(Boolean) : placed);

  // ---- Auto-hide ----
  // Showing and hiding are decided in Rust from the real mouse position (bar.rs
  // watch_autohide): page enter/leave events go missing when the bar moves under a still
  // pointer or a menu is open. Here only: slide in at once on hover, and stay out while editing.
  const autoHide = $derived(bar.autoHide && !bar.floating);
  const setBar = (f: (b: typeof bar) => void) => update((c) => f(c.profiles[c.activeProfile].bar));
  function onEnter() { if (autoHide) ipc.barReveal(true); }
  $effect(() => { if (autoHide && edit) ipc.barReveal(true); });

  onMount(() => {
    const offs = [
      events.editMode((on) => (edit = on)),
      win.onMoved(onMoved),
      // Alert sounds play here: the bar window is always alive.
      // After a screenshot / OCR snap: sound + a feedback card next to the Capture widget.
      events.captureDone((d) => {
        if (d.kind === 'shot') shutter(); else if (d.kind === 'ocr') tick();
        const el = document.querySelector('.capture-pill') ?? scroller;
        const h = d.kind === 'shot' ? 208 : d.kind === 'ocr' ? 168 : 100;
        setTimeout(() => ipc.popupOpen('__feedback', el.getBoundingClientRect(), 300, h, false), 60);
      }),
      // A notification was clicked: scroll to the widget that sent it, pulse a soft glow,
      // and (if set) open its panel. Removed since: the Notification Center instead.
      events.barJump(({ source, open }) => {
        const t = jumpTarget(source, shown);
        if (t.kind === 'none') return;
        const el = scroller.querySelectorAll<HTMLElement>(':scope > .slot')[t.index];
        if (!el) return;
        el.scrollIntoView({ behavior: 'smooth', inline: 'center', block: 'center' });
        el.classList.remove('jump'); void el.offsetWidth; el.classList.add('jump');
        setTimeout(() => el.classList.remove('jump'), 2300);
        if (t.kind === 'widget' && !open) return;
        setTimeout(() => {
          const w = shown[t.index];
          if (t.kind === 'widget' && t.part) {
            const seg = el.querySelectorAll<HTMLElement>('.seg')[PARTS.filter((p) => (w.settings as Record<string, unknown> | null)?.[p.show] !== false).findIndex((p) => p.key === t.part)] ?? el;
            const p = PARTS.find((x) => x.key === t.part)!;
            ipc.popupOpen(partPanel(p.key, w.instance), seg.getBoundingClientRect(), p.width, p.height);
          } else {
            const m = widgets[w.id];
            if (m?.Popup) openPopup(w.instance, el, m.Popup.width, m.Popup.height);
          }
        }, 450); // after the scroll settles
      }),
      events.soundPlay((s) => (s.kind === 'azan' ? playFile(s.src, s.short) : s.kind === 'adhan' ? playFile(adhanUrl, s.short) : s.kind === 'connect' || s.kind === 'disconnect' ? deviceTone(s.kind === 'connect', s.volume) : s.kind === 'plug' || s.kind === 'unplug' ? chargerTone(s.kind === 'plug', s.volume) : chime())),
    ];
    return () => offs.forEach((p) => p.then((off) => off()));
  });

  // ---- Move: drag the bar background, snap to an edge on release ----
  let dragging = false;
  let dragTimer: ReturnType<typeof setTimeout>;

  let dragFrom = { x: 0, y: 0 };

  async function onBarDown(e: PointerEvent) {
    if (e.button !== 0 || edit || (e.target as HTMLElement).closest('.slot, .handle, .edit-tools')) return;
    dragging = true;
    const p = await win.outerPosition();
    dragFrom = { x: p.x, y: p.y };
    await ipc.barDrag(true);
    win.startDragging();
    dragTimer = setTimeout(finishDrag, 250); // also ends a plain click
  }

  function onMoved() {
    if (!dragging) return; // ignore moves we made ourselves (apply_bar)
    clearTimeout(dragTimer);
    dragTimer = setTimeout(finishDrag, 250);
  }

  async function finishDrag() {
    // A pause mid-drag is not a drop: wait until the button is released.
    if (await ipc.mouseDown()) { dragTimer = setTimeout(finishDrag, 150); return; }
    dragging = false;
    await ipc.barDrag(false);
    const [pos, size, scale, mons] = await Promise.all([win.outerPosition(), win.outerSize(), win.scaleFactor(), availableMonitors()]);
    if (pos.x === dragFrom.x && pos.y === dragFrom.y) return; // a click, not a drag
    const rects = mons.map((m) => ({ x: m.position.x, y: m.position.y, w: m.size.width, h: m.size.height }));
    const s = snap({ x: pos.x, y: pos.y, w: size.width, h: size.height }, rects, 48 * scale);
    await update((c) => {
      const b = c.profiles[c.activeProfile].bar;
      b.monitor = s.monitor;
      b.autoHide = s.hide; // dragged past the edge = hide there; dragged back in = stay visible
      if (s.edge) { b.edge = s.edge; b.floating = null; } else { b.floating = { x: pos.x, y: pos.y }; }
    });
  }

  // ---- Resize: drag either end to change the bar's length ----
  // Only the window resizes while dragging (bar_preview); config is saved once on release.
  function onHandleDown(e: PointerEvent, side: 1 | -1) {
    if (e.button !== 0) return;
    const el = e.currentTarget as HTMLElement;
    el.setPointerCapture(e.pointerId);
    const start = vertical ? e.screenY : e.screenX;
    const startLen = bar.length;
    let span = 0;
    let length = startLen;
    Promise.all([availableMonitors(), primaryMonitor()]).then(([mons, primary]) => {
      const m = (bar.monitor !== null ? mons[bar.monitor] : primary) ?? mons[0];
      span = (vertical ? m.size.height : m.size.width) / m.scaleFactor;
    });
    const move = (ev: PointerEvent) => {
      if (!(ev.buttons & 1)) return done();
      if (!span) return;
      // The bar is centered, so it grows on both sides: double the pointer delta.
      const d = ((vertical ? ev.screenY : ev.screenX) - start) * side * 2;
      length = Math.min(1, Math.max(0.2, startLen + d / span));
      ipc.barPreview(length);
    };
    const done = () => {
      el.removeEventListener('pointermove', move);
      el.removeEventListener('lostpointercapture', done);
      if (length !== startLen) update((c) => { c.profiles[c.activeProfile].bar.length = length; });
    };
    el.addEventListener('pointermove', move);
    el.addEventListener('lostpointercapture', done); // fires on pointerup and on cancel
  }

  // ---- Scroll: wheel scrolls overflowing widgets with momentum ----
  $effect(() => {
    let v = 0, raf = 0;
    const step = () => {
      if (vertical) scroller.scrollTop += v; else scroller.scrollLeft += v;
      v *= 0.86;
      raf = Math.abs(v) > 0.3 ? requestAnimationFrame(step) : 0;
    };
    const onWheel = (e: WheelEvent) => {
      if (e.defaultPrevented) return;
      e.preventDefault();
      v += (Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY) * 0.3;
      if (!raf) raf = requestAnimationFrame(step);
    };
    scroller.addEventListener('wheel', onWheel, { passive: false });
    return () => { scroller.removeEventListener('wheel', onWheel); cancelAnimationFrame(raf); };
  });

  // ---- Widgets ----
  function onSlotClick(w: WidgetInst, el: HTMLElement) {
    if (swallowClick) { swallowClick = false; return; } // the click that ended a long-press
    const m = widgets[w.id];
    if (!edit && m.Popup) openPopup(w.instance, el, m.Popup.width, m.Popup.height);
  }

  async function onContext(e: MouseEvent) {
    e.preventDefault();
    await showMenu([
      { text: edit ? 'Done editing' : 'Edit widgets', action: () => ipc.setEditMode(!edit) },
      // Dock placement right here, no Settings window needed.
      { text: 'Position', items: (['top', 'bottom', 'left', 'right'] as const).map((e) => ({
        text: e[0].toUpperCase() + e.slice(1), checked: !bar.floating && bar.edge === e,
        action: () => setBar((b) => { b.edge = e; b.floating = null; }),
      })) },
      { text: 'Auto-hide', checked: autoHide, action: () => setBar((b) => { b.autoHide = !autoHide; b.floating = null; }) },
      await PredefinedMenuItem.new({ item: 'Separator' }),
      { text: 'Settings…', action: () => ipc.openSettings() },
      await PredefinedMenuItem.new({ item: 'Separator' }),
      { text: 'Task Manager', action: () => ipc.appLaunch('taskmgr.exe') },
    ]);
  }

  const removeWidget = (instance: string) =>
    update((c) => { const p = c.profiles[c.activeProfile]; p.widgets = p.widgets.filter((w) => w.instance !== instance); });

  const cycleSize = (w: WidgetInst) => {
    const sizes = widgets[w.id].sizes;
    const next: WidgetSize = sizes[(sizes.indexOf(w.size) + 1) % sizes.length];
    update((c) => { const t = c.profiles[c.activeProfile].widgets.find((x) => x.instance === w.instance); if (t) t.size = next; });
  };

  // Press and hold any widget (~0.6 s) to start editing, like the iPhone home screen.
  let holdTimer: ReturnType<typeof setTimeout>;
  let swallowClick = false;
  function startHold(e: PointerEvent) {
    if (edit || e.button !== 0) return;
    const x = e.clientX, y = e.clientY, el = e.currentTarget as HTMLElement;
    const cancel = () => { clearTimeout(holdTimer); el.removeEventListener('pointerup', cancel); el.removeEventListener('pointerleave', cancel); el.removeEventListener('pointermove', moved); };
    const moved = (m: PointerEvent) => { if (Math.hypot(m.clientX - x, m.clientY - y) > 6) cancel(); };
    holdTimer = setTimeout(() => { cancel(); swallowClick = true; ipc.setEditMode(true); }, 600);
    el.addEventListener('pointerup', cancel);
    el.addEventListener('pointerleave', cancel);
    el.addEventListener('pointermove', moved);
  }

  // ---- Edit mode: free drag ----
  // Hold ~250 ms (or start moving) to pick a widget up. It follows the pointer as a
  // floating copy; a placeholder marks where it will land and the others slide aside
  // (FLIP via animate:flip). Drop anywhere; Esc puts it back. The new order is saved on drop.
  let dragId = $state<string | null>(null);
  let ghost = $state<{ x: number; y: number; w: number; h: number } | null>(null);

  function onSlotDown(e: PointerEvent, instance: string) {
    startHold(e);
    if (!edit || e.button !== 0 || (e.target as HTMLElement).closest('.badge')) return;
    const el = e.currentTarget as HTMLElement;
    // Track the pointer on the window, not with pointer capture on the slot: reordering
    // moves the slot's DOM node, which silently drops capture and ended the drag after
    // the first swap.
    const start = { x: e.clientX, y: e.clientY };
    const before = placed.map((w) => w.instance).join();
    let grab = { dx: 0, dy: 0 };
    let last = e;
    let raf = 0;
    let picked = false;

    const pick = () => {
      if (picked) return;
      picked = true;
      const r = el.getBoundingClientRect();
      grab = { dx: start.x - r.left, dy: start.y - r.top };
      ghost = { x: r.left, y: r.top, w: r.width, h: r.height };
      dragId = instance;
      order = placed.map((w) => w.instance);
    };
    const pickTimer = setTimeout(pick, 250);

    const frame = () => {
      raf = 0;
      if (!ghost || !order) return;
      ghost = { ...ghost, x: last.clientX - grab.dx, y: vertical ? last.clientY - grab.dy : ghost.y };
      const others = [...scroller.querySelectorAll<HTMLElement>('.slot:not(.placeholder)')]
        .map((s) => { const r = s.getBoundingClientRect(); return vertical ? r.top + r.height / 2 : r.left + r.width / 2; });
      const to = insertIndex(others, vertical ? last.clientY : last.clientX);
      const rest = order.filter((id) => id !== instance);
      const next = [...rest.slice(0, to), instance, ...rest.slice(to)];
      if (next.join() !== order.join()) order = next;
    };
    const move = (ev: PointerEvent) => {
      last = ev;
      if (!picked && Math.hypot(ev.clientX - start.x, ev.clientY - start.y) > 6) { clearTimeout(pickTimer); pick(); }
      if (picked && !raf) raf = requestAnimationFrame(frame);
    };
    const reset = () => {
      clearTimeout(pickTimer);
      cancelAnimationFrame(raf);
      removeEventListener('pointermove', move);
      removeEventListener('pointerup', up);
      removeEventListener('pointercancel', up);
      removeEventListener('keydown', onKey);
      dragId = null;
      ghost = null;
    };
    const onKey = (k: KeyboardEvent) => {
      if (k.key !== 'Escape') return;
      order = null; // back to where it was
      reset();
    };
    const up = async () => {
      const final = order;
      reset();
      try {
        if (picked && final && final.join() !== before) {
          await update((c) => {
            const p = c.profiles[c.activeProfile];
            p.widgets = final.map((id) => p.widgets.find((w) => w.instance === id)!).filter(Boolean)
              .concat(p.widgets.filter((w) => !final.includes(w.instance)));
          });
        }
      } finally {
        order = null;
      }
    };
    addEventListener('pointermove', move);
    addEventListener('pointerup', up);
    addEventListener('pointercancel', up);
    addEventListener('keydown', onKey);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="surface bar dock" class:vertical class:edit onpointerdown={onBarDown} oncontextmenu={onContext}
     onpointerenter={onEnter} data-size={bar.size} data-side={bar.sideWidth ?? 'regular'}>
  <div class="handle start" onpointerdown={(e) => onHandleDown(e, -1)}></div>

  <div class="scroller" bind:this={scroller}>
    {#each shown as w (w.instance)}
      {@const m = widgets[w.id]}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="slot"
        class:placeholder={dragId === w.instance}
        role="button"
        tabindex="-1"
        animate:flip={{ duration: 280 }}
        onpointerdown={(e) => onSlotDown(e, w.instance)}
        onclickcapture={(e) => { if (swallowClick) { swallowClick = false; e.stopPropagation(); e.preventDefault(); } }}
        onclick={(e) => onSlotClick(w, e.currentTarget as HTMLElement)}
      >
        <m.Pill instance={w.instance} size={vertical ? 'small' : w.size} settings={settingsOf(m, w)} {vertical} />
        {#if edit}
          <button class="badge remove" title="Remove" onclick={() => removeWidget(w.instance)}><X size={11} strokeWidth={3} /></button>
          {#if m.sizes.length > 1}
            <button class="badge resize" title="Size: {w.size}" onclick={() => cycleSize(w)}><Maximize2 size={10} strokeWidth={3} /></button>
          {/if}
        {/if}
      </div>
    {/each}
    {#if placed.length === 0}
      <span class="dim empty">Right-click → Edit widgets</span>
    {/if}
  </div>

  {#if ghost && dragId}
    {@const w = placed.find((x) => x.instance === dragId)}
    {#if w}
      {@const m = widgets[w.id]}
      <div class="ghost" style:width="{ghost.w}px" style:height="{ghost.h}px" style:transform="translate({ghost.x}px, {ghost.y}px) scale(1.06)">
        <m.Pill instance={w.instance} size={vertical ? 'small' : w.size} settings={settingsOf(m, w)} {vertical} />
      </div>
    {/if}
  {/if}

  {#if edit}
    <div class="edit-tools">
      <button class="icon-btn" title="Add widget" onclick={(e) => openPopup(GALLERY, e.currentTarget, 340, 460)}><Plus size={16} /></button>
      <button class="icon-btn done" title="Done" onclick={() => ipc.setEditMode(false)}><Check size={16} /></button>
    </div>
  {/if}

  <div class="handle end" onpointerdown={(e) => onHandleDown(e, 1)}></div>
</div>

<style>
  .bar {
    display: flex;
    align-items: stretch;
    gap: var(--gap);
    padding: 5px;
    position: relative;
  }
  .bar.vertical { flex-direction: column; }
  .bar[data-size='compact'] { padding: 4px; }

  .scroller {
    flex: 1;
    display: flex;
    align-items: stretch;
    /* Centred with equal space both sides; 'safe' falls back to start when the
       widgets overflow, so the first one stays reachable by scrolling. */
    justify-content: safe center;
    gap: var(--gap);
    overflow: hidden;
    min-width: 0;
    min-height: 0;
  }
  .vertical .scroller { flex-direction: column; }

  /* Jumped to from a notification: a soft accent ring pulses 3 times (~2 s), then fades. */
  .slot:global(.jump) > :global(*) { animation: jumpglow 0.7s ease-in-out 3; border-radius: var(--radius-pill); }
  @keyframes jumpglow { 50% { box-shadow: 0 0 0 2px var(--accent), 0 0 16px color-mix(in srgb, var(--accent) 65%, transparent); } }
  /* where the dragged widget will land */
  .slot.placeholder { animation: none !important; border-radius: var(--radius-pill); outline: 2px dashed color-mix(in srgb, var(--accent) 70%, transparent); outline-offset: -2px; }
  .slot.placeholder > :global(*) { opacity: 0; }
  .ghost {
    position: fixed; left: 0; top: 0; z-index: 10; pointer-events: none; display: flex;
    border-radius: var(--radius-pill); box-shadow: 0 10px 26px rgba(0, 0, 0, 0.35);
    transition: transform 40ms linear; will-change: transform;
  }
  /* Slot and left/right column rules are shared with the Settings preview: glass.css (.dock). */
  .edit .slot { animation: jiggle 0.28s ease-in-out infinite; }
  .edit .slot:nth-child(2n) { animation-delay: -0.14s; }

  .badge {
    position: absolute;
    width: 18px; height: 18px;
    display: grid; place-items: center;
    border-radius: 50%;
    color: #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
    z-index: 2;
  }
  .remove { top: -3px; left: -3px; background: #ff453a; }
  .resize { bottom: -3px; right: -3px; background: var(--accent); }

  .edit-tools { display: flex; gap: 2px; align-items: center; }
  .vertical .edit-tools { flex-direction: column; }
  .done { background: var(--accent); color: #fff; }

  .handle { flex: none; width: 6px; cursor: ew-resize; border-radius: 3px; }
  .vertical .handle { width: auto; height: 6px; cursor: ns-resize; }
  .handle:hover { background: var(--pill-hover); }

  .empty { align-self: center; padding: 0 12px; }
</style>
