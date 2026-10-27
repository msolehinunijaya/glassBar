<script lang="ts">
  import { showMenu, type Item } from '../../lib/menu';
  import { onMount } from 'svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { events, ipc, type AppWindow } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import { flip } from 'svelte/animate';
  import { slide, type SlideParams } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  // Icons that appear or go (hover shows the ones behind "+N", apps open or close): slide
  // open along the dock and fade in, the reverse when collapsing.
  function reveal(node: Element, o: SlideParams) {
    const s = slide(node, o);
    return { ...s, css: (t: number, u: number) => `${s.css?.(t, u) ?? ''};opacity:${t}` };
  }
  import { insertIndex } from '../../lib/geometry';
  import { group, isWebApp, keyFor, keyOf, limitIcons, moveKey, nextWindow, pinsInOrder, stableOrder, type Group } from './groups';
  import type { AppsSettings } from './index';
  import StartButtons from '../start/StartButtons.svelte';
  import { ADD_APP, APP_DRAG } from '../folders/folders';

  let { instance, size, settings, vertical }: PillProps<AppsSettings> = $props();
  let windows = $state<AppWindow[]>([]);
  let icons = $state<Record<string, string | null>>({});
  // Icons keep their place: the order you dragged them into (saved), then first appearance.
  // Windows inside a group are ordered by id.
  let seen: string[] = [];
  let liveOrder = $state<string[] | null>(null); // while dragging
  const groups = $derived.by(() => {
    const raw = group(settings.pinned, windows);
    seen = stableOrder(seen.length ? seen : settings.order, raw.map((g) => g.key));
    const byKey = new Map(raw.map((g) => [g.key, { ...g, windows: [...g.windows].sort((a, b) => a.id - b.id) }]));
    return (liveOrder ?? seen).filter((k) => byKey.has(k)).map((k) => byKey.get(k)!);
  });
  // Pinned apps whose program was uninstalled or moved.
  let missing = $state<Record<string, boolean>>({});
  let failed = $state<string | null>(null);
  $effect(() => {
    const paths = settings.pinned.map((p) => p.path);
    ipc.pathsExist(paths).then((ok) => { missing = Object.fromEntries(paths.map((p, i) => [keyOf(p), !ok[i]])); }).catch(() => {});
  });

  let edit = false;
  onMount(() => { const off = events.editMode((on) => (edit = on)); return () => { off.then((f) => f()); }; });

  // ---- Drag to reorder (hold ~0.3 s or just drag). The icon moves to its new place
  // live and the others slide aside; release saves. It never launches anything. ----
  let dragKey = $state<string | null>(null);
  // Show at most `maxIcons` (the rest as "+N") until the pointer is on the launcher.
  let expanded = $state(false);
  let collapseTimer: ReturnType<typeof setTimeout> | undefined;
  const expand = () => { clearTimeout(collapseTimer); expanded = true; };
  const collapse = () => { clearTimeout(collapseTimer); collapseTimer = setTimeout(() => (expanded = false), 400); };
  // Every icon stays in the row; the ones behind "+N" fold to zero width (a CSS transition),
  // so expanding and collapsing both animate in place. (Removing them instead made the leaving
  // icons float over the next widgets while the row shrank.)
  const limited = $derived(expanded || dragKey ? { shown: groups, hidden: 0 } : limitIcons(groups, settings.maxIcons ?? 0, (g) => g.windows.some((w) => w.active)));
  const visibleKeys = $derived(new Set(limited.shown.map((g) => g.key)));
  let swallow = false;
  let row: HTMLElement;
  function onDown(e: PointerEvent, g: Group) {
    if (edit || e.button !== 0) return; // in edit mode the whole widget drags instead
    e.stopPropagation(); // the bar's press-and-hold-to-edit stays out of the launcher
    const axis = (ev: PointerEvent) => (vertical ? ev.clientY : ev.clientX);
    const start = axis(e);
    let picked = false;
    const pick = () => {
      if (picked) return;
      picked = true;
      clearTimeout(hoverTimer);
      if (previewOpen) { previewOpen = false; ipc.popupHide(); }
      dragKey = g.key;
      liveOrder = groups.map((x) => x.key);
      document.documentElement.dataset[APP_DRAG] = '1'; // folders on the dock light up
    };
    const timer = setTimeout(pick, 300);
    const move = (ev: PointerEvent) => {
      if (!picked && Math.abs(axis(ev) - start) > 6) pick();
      if (!picked || !liveOrder) return;
      const others = [...row.querySelectorAll<HTMLElement>('.app:not(.dragging)')].map((b) => {
        const r = b.getBoundingClientRect();
        return vertical ? r.top + r.height / 2 : r.left + r.width / 2;
      });
      const next = moveKey(liveOrder, g.key, insertIndex(others, axis(ev)));
      if (next.join() !== liveOrder.join()) liveOrder = next;
    };
    const up = (ev?: PointerEvent) => {
      clearTimeout(timer);
      removeEventListener('pointermove', move);
      removeEventListener('pointerup', up);
      removeEventListener('keydown', esc);
      delete document.documentElement.dataset[APP_DRAG];
      if (!picked) return;
      swallow = true;
      // Released on an App Folders tile: the app goes into that folder, the row stays as it was.
      const folder = ev && document.elementFromPoint(ev.clientX, ev.clientY)?.closest<HTMLElement>('[data-folder-drop]');
      if (folder) {
        folder.dispatchEvent(new CustomEvent(ADD_APP, { bubbles: true, detail: { path: g.path, args: g.pin?.args, aumid: g.aumid, ...(isWebApp(g.aumid) ? { name: g.name } : {}) } }));
      } else if (liveOrder) saveOrder(liveOrder);
      dragKey = null;
      liveOrder = null;
    };
    const esc = (k: KeyboardEvent) => { if (k.key === 'Escape') { liveOrder = null; picked = false; dragKey = null; up(); } };
    addEventListener('pointermove', move);
    addEventListener('pointerup', up);
    addEventListener('keydown', esc);
  }
  function saveOrder(order: string[]) {
    seen = order;
    updateWidgetSettings(instance, { order, pinned: pinsInOrder(settings.pinned, order) });
  }
  function nudge(g: Group, by: number) {
    const order = groups.map((x) => x.key);
    saveOrder(moveKey(order, g.key, order.indexOf(g.key) + by));
  }
  async function addApp() {
    const t = await ipc.appBrowse().catch(() => null);
    if (t && !settings.pinned.some((p) => keyOf(p.path) === keyOf(t.path))) setPins([...settings.pinned, { path: t.path, args: t.args || undefined }]);
  }
  const iconSize = $derived(size === 'small' ? 22 : 26);

  onMount(() => {
    ipc.appsList().then((w) => (windows = w));
    const off = events.appsChanged((w) => (windows = w));
    return () => { off.then((f) => f()); };
  });

  // Fetch each program's real icon (Store icon, program file, or window). If none was found
  // yet (e.g. a window still starting), ask again when the window list changes, at most
  // every 5 s, instead of settling for the letter placeholder.
  const asked = new Map<string, number>();
  $effect(() => {
    const now = Date.now();
    for (const g of groups) {
      if (icons[g.key] || now - (asked.get(g.key) ?? -Infinity) < 5000) continue;
      asked.set(g.key, now);
      ipc.appIcon(g.path, g.windows[0]?.id ?? null, g.aumid ?? null).then((src) => { if (src) icons[g.key] = src; });
    }
  });
  const launch = (g: Group, args = g.pin?.args ?? '') => ipc.appLaunch(g.path, args, g.aumid ?? null);

  // Hovering a running app shows live previews of its windows (without taking keyboard focus).
  let hoverTimer: ReturnType<typeof setTimeout>;
  let previewOpen = false;
  function onHover(e: PointerEvent, g: Group) {
    clearTimeout(hoverTimer);
    if (!g.windows.length || dragKey) return;
    const el = e.currentTarget as HTMLElement;
    const n = Math.min(g.windows.length, 6);
    hoverTimer = setTimeout(() => {
      previewOpen = true;
      ipc.popupOpen(`__windows:${g.key}`, el.getBoundingClientRect(), 16 + n * 216, 172, false);
    }, 300);
  }
  function onUnhover() {
    clearTimeout(hoverTimer);
    ipc.popupLeave();
  }

  function open(g: Group) {
    // A click always closes the hover preview first (no box left behind).
    clearTimeout(hoverTimer);
    if (previewOpen) { previewOpen = false; ipc.popupHide(); }
    if (swallow) { swallow = false; return; } // the click that ended a drag
    const w = nextWindow(g);
    if (w) ipc.appActivate(w.id);
    else launch(g).catch(() => {
      failed = g.key;
      setTimeout(() => { if (failed === g.key) failed = null; }, 4000);
    });
  }

  const setPins = (pinned: AppsSettings['pinned']) => updateWidgetSettings(instance, { pinned });

  async function menu(e: MouseEvent, g: Group) {
    e.preventDefault();
    e.stopPropagation();
    const items: Item[] = [
      { text: 'New window', action: () => launch(g) },
      g.pin
        ? { text: 'Unpin', action: () => setPins(settings.pinned.filter((p) => keyFor(p.path, p.aumid) !== g.key)) }
        : { text: 'Pin to bar', action: () => setPins([...settings.pinned, { path: g.path, aumid: g.aumid, ...(isWebApp(g.aumid) ? { name: g.name } : {}) }]) },
    ];
    const i = groups.indexOf(g);
    if (i > 0) items.push({ text: vertical ? 'Move up' : 'Move left', action: () => nudge(g, -1) });
    if (i < groups.length - 1) items.push({ text: vertical ? 'Move down' : 'Move right', action: () => nudge(g, 1) });
    items.push({ text: 'Pin another app…', action: addApp });
    if (g.windows.length) {
      items.push({ text: g.windows.length > 1 ? `Close all ${g.windows.length} windows` : 'Close window',
        action: () => { g.windows.forEach((w) => ipc.appClose(w.id)); } });
    }
    // Recent files that open with this app (best effort; Windows hides real jump lists).
    const recent = await Promise.race([
      ipc.jumplist(g.path).catch(() => []),
      new Promise<[]>((ok) => setTimeout(() => ok([]), 400)),
    ]);
    if (recent.length) {
      items.unshift(
        ...recent.map((r, i) => ({ text: r.name, action: () => ipc.appLaunch(r.path) })),
        '-',
      );
    }
    await showMenu(items);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="pill apps" class:vertical bind:this={row} onpointerenter={expand} onpointerleave={collapse}>
  {#if settings.startPlacement === 'inline'}
    <StartButtons searchStyle={settings.searchStyle} color={settings.startColor} custom={settings.startCustom} />
    <span class="sep"></span>
  {/if}
  {#each groups as g (g.key)}
    {@const active = g.windows.some((w) => w.active)}
    {@const gone = !g.windows.length && missing[g.key]}
    <button
      class="app"
      class:active
      class:dragging={dragKey === g.key}
      class:gone
      class:failed={failed === g.key}
      animate:flip={{ duration: 220 }}
      class:hid={!visibleKeys.has(g.key)}
      aria-hidden={!visibleKeys.has(g.key)}
      tabindex={visibleKeys.has(g.key) ? 0 : -1}
      in:reveal={{ axis: vertical ? 'y' : 'x', duration: 240, easing: cubicOut }}
      title={failed === g.key ? `Couldn't start ${g.name}. Right-click to unpin.` : gone ? `${g.name} wasn't found at ${g.path}. Right-click to unpin.`
        : g.windows.length ? g.windows.map((w) => w.title).join('\n') : `${g.name} (pinned)`}
      onpointerdown={(e) => onDown(e, g)}
      onclick={(e) => { e.stopPropagation(); open(g); }}
      onauxclick={(e) => { if (e.button === 1) launch(g); }}
      oncontextmenu={(e) => menu(e, g)}
      onpointerenter={(e) => onHover(e, g)}
      onpointerleave={onUnhover}
    >
      {#if icons[g.key]}
        <img src={icons[g.key]} alt="" width={iconSize} height={iconSize} draggable="false" />
      {:else}
        <span class="fallback" style:width="{iconSize}px" style:height="{iconSize}px">{g.name[0]}</span>
      {/if}
      {#if size === 'large' && active && settings.showTitles}
        <span class="title ellipsis">{g.windows.find((w) => w.active)?.title}</span>
      {/if}
      {#if g.windows.length}
        <span class="dots">{#each g.windows.slice(0, 3) as w (w.id)}<i></i>{/each}</span>
      {/if}
    </button>
  {/each}
  {#if limited.hidden}
    <button class="more-apps" transition:reveal={{ axis: vertical ? 'y' : 'x', duration: 280, easing: cubicOut }} title="{limited.hidden} more apps" onpointerdown={(e) => e.stopPropagation()} onclick={(e) => { e.stopPropagation(); expand(); }}>+{limited.hidden}</button>
  {/if}
  {#if groups.length === 0}<span class="dim">No apps</span>{/if}
  {#if settings.startPlacement === 'after'}
    <span class="sep"></span>
    <StartButtons searchStyle={settings.searchStyle} color={settings.startColor} custom={settings.startCustom} />
  {/if}
</div>

<style>
  .apps { gap: 2px; padding: 0 4px; }
  .sep { width: 1px; align-self: stretch; margin: 8px 3px; background: var(--line-soft); flex: none; }
  .vertical .sep { width: auto; height: 1px; margin: 3px 8px; }
  .apps.vertical { flex-direction: column; height: auto; padding: 4px 0; }
  .app {
    position: relative; display: flex; align-items: center; gap: 6px;
    height: 100%; padding: 0 6px; border-radius: calc(10px * var(--rs));
    max-width: 400px; max-height: 400px;
    overflow: hidden; /* while a slot widens or folds, its icon doesn't spill over the next one */
    transition: background 0.2s var(--ease), transform 0.3s var(--spring),
      max-width 0.28s var(--ease), max-height 0.28s var(--ease), padding 0.28s var(--ease), margin 0.28s var(--ease), opacity 0.22s var(--ease);
  }
  /* An icon is ~40 px: fold from about that, so the motion is even (from 400 px it would sit
     still for most of the time). Only the active app shows a title, and it's never folded. */
  .app:not(.active) { max-width: 48px; }
  .vertical .app:not(.active) { max-width: 400px; max-height: 48px; }
  /* Behind "+N": folded away (the row's 2 px gap is taken back too). */
  .app.hid { max-width: 0; padding-left: 0; padding-right: 0; margin-right: -2px; opacity: 0; overflow: hidden; pointer-events: none; }
  .vertical .app.hid { max-width: 400px; max-height: 0; padding-top: 0; padding-bottom: 0; margin-right: 0; margin-bottom: -2px; }
  .vertical .app { height: auto; padding: 5px; }
  .app:hover { background: var(--pill-hover); }
  .app:active { transform: scale(0.88); }
  .app.active { background: var(--pill-hover); }
  img { display: block; }
  .fallback { display: grid; place-items: center; border-radius: calc(8px * var(--rs)); background: var(--accent); color: var(--on-accent); font-weight: 600; }
  .title { max-width: 140px; font-size: 12px; }
  .dots { position: absolute; bottom: 1px; left: 50%; transform: translateX(-50%); display: flex; gap: 2px; }
  .dots i { width: 4px; height: 4px; border-radius: calc(2px * var(--rs)); background: var(--fg-dim); }
  .active .dots i:first-child { width: 12px; background: var(--accent); }
  .app.dragging { z-index: 2; transform: scale(1.18); background: var(--pill-hover); box-shadow: 0 6px 18px rgba(0,0,0,0.25); cursor: grabbing; }
  .app.gone img, .app.gone .fallback { opacity: 0.35; filter: grayscale(1); }
  .app.failed { box-shadow: inset 0 0 0 2px #ff453a; }
  .more-apps {
    flex: none; align-self: center; min-width: 26px; height: 22px; padding: 0 6px; border-radius: var(--r-chip, 11px);
    background: var(--pill-hover); color: var(--fg); font-size: 11px; font-weight: 600; font-variant-numeric: tabular-nums;
  }
  .more-apps:hover { background: var(--pill-active, var(--pill-hover)); }
</style>
