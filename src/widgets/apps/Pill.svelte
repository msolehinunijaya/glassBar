<script lang="ts">
  import { showMenu, type Item } from '../../lib/menu';
  import { onMount } from 'svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { events, ipc, type AppWindow } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import { flip } from 'svelte/animate';
  import { insertIndex } from '../../lib/geometry';
  import { group, keyOf, moveKey, nextWindow, pinsInOrder, stableOrder, type Group } from './groups';
  import type { AppsSettings } from './index';
  import StartButtons from '../start/StartButtons.svelte';

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
    const up = () => {
      clearTimeout(timer);
      removeEventListener('pointermove', move);
      removeEventListener('pointerup', up);
      removeEventListener('keydown', esc);
      if (!picked) return;
      swallow = true;
      if (liveOrder) saveOrder(liveOrder);
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
        ? { text: 'Unpin', action: () => setPins(settings.pinned.filter((p) => keyOf(p.path) !== g.key)) }
        : { text: 'Pin to bar', action: () => setPins([...settings.pinned, { path: g.path, aumid: g.aumid }]) },
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

<div class="pill apps" class:vertical bind:this={row}>
  {#if settings.startPlacement === 'inline'}
    <StartButtons searchStyle={settings.searchStyle} />
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
  {#if groups.length === 0}<span class="dim">No apps</span>{/if}
</div>

<style>
  .apps { gap: 2px; padding: 0 4px; }
  .sep { width: 1px; align-self: stretch; margin: 8px 3px; background: var(--line-soft); flex: none; }
  .vertical .sep { width: auto; height: 1px; margin: 3px 8px; }
  .apps.vertical { flex-direction: column; height: auto; padding: 4px 0; }
  .app {
    position: relative; display: flex; align-items: center; gap: 6px;
    height: 100%; padding: 0 6px; border-radius: 10px;
    transition: background 0.2s var(--ease), transform 0.3s var(--spring);
  }
  .vertical .app { height: auto; padding: 5px; }
  .app:hover { background: var(--pill-hover); }
  .app:active { transform: scale(0.88); }
  .app.active { background: var(--pill-hover); }
  img { display: block; }
  .fallback { display: grid; place-items: center; border-radius: 8px; background: var(--accent); color: #fff; font-weight: 600; }
  .title { max-width: 140px; font-size: 12px; }
  .dots { position: absolute; bottom: 1px; left: 50%; transform: translateX(-50%); display: flex; gap: 2px; }
  .dots i { width: 4px; height: 4px; border-radius: 2px; background: var(--fg-dim); }
  .active .dots i:first-child { width: 12px; background: var(--accent); }
  .app.dragging { z-index: 2; transform: scale(1.18); background: var(--pill-hover); box-shadow: 0 6px 18px rgba(0,0,0,0.25); cursor: grabbing; }
  .app.gone img, .app.gone .fallback { opacity: 0.35; filter: grayscale(1); }
  .app.failed { box-shadow: inset 0 0 0 2px #ff453a; }
</style>
