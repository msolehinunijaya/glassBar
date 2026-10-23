<script lang="ts">
  import { PredefinedMenuItem } from '@tauri-apps/api/menu';
  import { showMenu } from '../../lib/menu';
  import { onMount } from 'svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { events, ipc, type AppWindow } from '../../lib/ipc';
  import type { PillProps } from '../types';
  import { group, keyOf, nextWindow, stableOrder, type Group } from './groups';
  import type { AppsSettings } from './index';

  let { instance, size, settings, vertical }: PillProps<AppsSettings> = $props();
  let windows = $state<AppWindow[]>([]);
  let icons = $state<Record<string, string | null>>({});
  // Icons keep their place: order by first appearance, and windows inside a group by id.
  let seen: string[] = [];
  const groups = $derived.by(() => {
    const raw = group(settings.pinned, windows);
    seen = stableOrder(seen, raw.map((g) => g.key));
    const byKey = new Map(raw.map((g) => [g.key, { ...g, windows: [...g.windows].sort((a, b) => a.id - b.id) }]));
    return seen.map((k) => byKey.get(k)!);
  });
  const iconSize = $derived(size === 'small' ? 22 : 26);

  onMount(() => {
    ipc.appsList().then((w) => (windows = w));
    const off = events.appsChanged((w) => (windows = w));
    return () => { off.then((f) => f()); };
  });

  // Fetch each program's icon once.
  const requested = new Set<string>();
  $effect(() => {
    for (const g of groups) {
      if (requested.has(g.key)) continue;
      requested.add(g.key);
      ipc.appIcon(g.path, g.windows[0]?.id ?? null).then((src) => (icons[g.key] = src));
    }
  });

  // Hovering an app with 2+ windows opens a chooser (without taking keyboard focus).
  let hoverTimer: ReturnType<typeof setTimeout>;
  function onHover(e: PointerEvent, g: Group) {
    clearTimeout(hoverTimer);
    if (g.windows.length < 2) return;
    const el = e.currentTarget as HTMLElement;
    hoverTimer = setTimeout(() => {
      ipc.popupOpen(`__windows:${g.key}`, el.getBoundingClientRect(), 300, Math.min(420, 48 + g.windows.length * 38), false);
    }, 350);
  }
  function onUnhover() {
    clearTimeout(hoverTimer);
    ipc.popupLeave();
  }

  function open(g: Group) {
    const w = nextWindow(g);
    if (w) ipc.appActivate(w.id);
    else ipc.appLaunch(g.path, g.pin?.args ?? '');
  }

  const setPins = (pinned: AppsSettings['pinned']) => updateWidgetSettings(instance, { pinned });

  async function menu(e: MouseEvent, g: Group) {
    e.preventDefault();
    e.stopPropagation();
    const items: ({ id: string; text: string; action: () => unknown } | PredefinedMenuItem)[] = [
      { id: 'app:new', text: 'New window', action: () => ipc.appLaunch(g.path, g.pin?.args ?? '') },
      g.pin
        ? { id: 'app:unpin', text: 'Unpin', action: () => setPins(settings.pinned.filter((p) => keyOf(p.path) !== g.key)) }
        : { id: 'app:pin', text: 'Pin to bar', action: () => setPins([...settings.pinned, { path: g.path }]) },
    ];
    if (g.windows.length) {
      items.push({ id: 'app:close', text: g.windows.length > 1 ? `Close all ${g.windows.length} windows` : 'Close window',
        action: () => { g.windows.forEach((w) => ipc.appClose(w.id)); } });
    }
    // Recent files that open with this app (best effort; Windows hides real jump lists).
    const recent = await Promise.race([
      ipc.jumplist(g.path).catch(() => []),
      new Promise<[]>((ok) => setTimeout(() => ok([]), 400)),
    ]);
    if (recent.length) {
      items.unshift(
        ...recent.map((r, i) => ({ id: `app:recent-${i}`, text: r.name, action: () => ipc.appLaunch(r.path) })),
        await PredefinedMenuItem.new({ item: 'Separator' }),
      );
    }
    await showMenu(items);
  }
</script>

<div class="pill apps" class:vertical>
  {#each groups as g (g.key)}
    {@const active = g.windows.some((w) => w.active)}
    <button
      class="app"
      class:active
      title={g.windows.length ? g.windows.map((w) => w.title).join('\n') : g.name}
      onclick={(e) => { e.stopPropagation(); open(g); }}
      onauxclick={(e) => { if (e.button === 1) ipc.appLaunch(g.path, g.pin?.args ?? ''); }}
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
</style>
