<script lang="ts">
  import { availableMonitors, getCurrentWindow, primaryMonitor } from '@tauri-apps/api/window';
  import { Menu } from '@tauri-apps/api/menu';
  import { flip } from 'svelte/animate';
  import { onMount } from 'svelte';
  import { Check, Maximize2, Plus, X } from '@lucide/svelte';
  import { profile, store, update } from '../lib/config.svelte';
  import { events, ipc, type WidgetInst, type WidgetSize } from '../lib/ipc';
  import { indexAt, reorder, snap } from '../lib/geometry';
  import { GALLERY, openPopup } from '../lib/popup';
  import { chime, playFile } from '../lib/sound';
  import { settingsOf, widgets } from '../widgets/registry';

  const win = getCurrentWindow();
  let edit = $state(false);
  let scroller: HTMLDivElement;

  const bar = $derived(profile().bar);
  const vertical = $derived(bar.edge === 'left' || bar.edge === 'right');
  const placed = $derived(profile().widgets.filter((w) => widgets[w.id]));

  // While dragging a widget in edit mode we show a local order, committed on drop.
  let order = $state<string[] | null>(null);
  const shown = $derived(order ? order.map((id) => placed.find((w) => w.instance === id)!).filter(Boolean) : placed);

  onMount(() => {
    const offs = [
      events.editMode((on) => (edit = on)),
      win.onMoved(onMoved),
      // Alert sounds play here: the bar window is always alive.
      events.soundPlay((s) => (s.kind === 'azan' ? playFile(s.src) : chime())),
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
    const m = widgets[w.id];
    if (!edit && m.Popup) openPopup(w.instance, el, m.Popup.width, m.Popup.height);
  }

  async function onContext(e: MouseEvent) {
    e.preventDefault();
    const menu = await Menu.new({
      items: [
        { id: 'edit', text: edit ? 'Done editing' : 'Edit widgets', action: () => ipc.setEditMode(!edit) },
        { id: 'settings', text: 'Settings…', action: () => ipc.openSettings() },
      ],
    });
    await menu.popup();
    await menu.close(); // free the native menu resource
  }

  const removeWidget = (instance: string) =>
    update((c) => { const p = c.profiles[c.activeProfile]; p.widgets = p.widgets.filter((w) => w.instance !== instance); });

  const cycleSize = (w: WidgetInst) => {
    const sizes = widgets[w.id].sizes;
    const next: WidgetSize = sizes[(sizes.indexOf(w.size) + 1) % sizes.length];
    update((c) => { const t = c.profiles[c.activeProfile].widgets.find((x) => x.instance === w.instance); if (t) t.size = next; });
  };

  // Edit-mode reorder: press a widget, slide it, release.
  function onSlotDown(e: PointerEvent, instance: string) {
    if (!edit || e.button !== 0 || (e.target as HTMLElement).closest('.badge')) return;
    const el = e.currentTarget as HTMLElement;
    el.setPointerCapture(e.pointerId);
    order = placed.map((w) => w.instance);
    const move = (ev: PointerEvent) => {
      const slots = [...scroller.querySelectorAll<HTMLElement>('.slot')];
      const centers = slots.map((s) => { const r = s.getBoundingClientRect(); return vertical ? r.top + r.height / 2 : r.left + r.width / 2; });
      const to = indexAt(centers, vertical ? ev.clientY : ev.clientX);
      const from = order!.indexOf(instance);
      if (to !== from) order = reorder(order!, from, to);
    };
    const before = order.join();
    const up = async () => {
      el.removeEventListener('pointermove', move);
      el.removeEventListener('lostpointercapture', up);
      const final = order!;
      try {
        if (final.join() !== before) {
          await update((c) => {
            const p = c.profiles[c.activeProfile];
            p.widgets = final.map((id) => p.widgets.find((w) => w.instance === id)!).concat(p.widgets.filter((w) => !final.includes(w.instance)));
          });
        }
      } finally {
        order = null;
      }
    };
    el.addEventListener('pointermove', move);
    el.addEventListener('lostpointercapture', up);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="surface bar" class:vertical class:edit onpointerdown={onBarDown} oncontextmenu={onContext} data-size={bar.size}>
  <div class="handle start" onpointerdown={(e) => onHandleDown(e, -1)}></div>

  <div class="scroller" bind:this={scroller}>
    {#each shown as w (w.instance)}
      {@const m = widgets[w.id]}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="slot"
        role="button"
        tabindex="-1"
        animate:flip={{ duration: 280 }}
        onpointerdown={(e) => onSlotDown(e, w.instance)}
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
    gap: var(--gap);
    overflow: hidden;
    min-width: 0;
    min-height: 0;
  }
  .vertical .scroller { flex-direction: column; }

  .slot { position: relative; flex: none; display: flex; }
  .vertical .slot { justify-content: center; }
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
