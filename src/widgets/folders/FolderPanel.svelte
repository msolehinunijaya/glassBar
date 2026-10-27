<script lang="ts">
  // One folder, opened from the dock: its name (click to rename; empty = no label) and its
  // apps in a 4-column grid. Click an app to open it (or switch to its window). Drag to
  // reorder; drag one out of the grid to take it out of the folder. + adds a program or
  // shortcut, and so does dropping one from Explorer onto the panel.
  import { Plus } from '@lucide/svelte';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { ipc } from '../../lib/ipc';
  import { profile, updateWidgetSettings } from '../../lib/config.svelte';
  import { showMenu } from '../../lib/menu';
  import { nextWindow } from '../apps/groups';
  import type { Pin } from '../apps/index';
  import type { FoldersSettings } from './index';
  import { addApps, isApp, moveApp, pinKey, pinName, removeApp, replace, type Folder } from './folders';
  import { iconOf, liveWindows } from './appIcons.svelte';

  let { instance, id, close }: { instance: string; id: string; close: () => void } = $props();
  const settings = $derived(profile().widgets.find((w) => w.instance === instance)?.settings as FoldersSettings | undefined);
  const folder = $derived(settings?.folders?.find((f) => f.id === id));
  const save = (f: Folder) => settings && updateWidgetSettings(instance, { folders: replace(settings.folders, f) });
  const wins = liveWindows();

  function launch(p: Pin) {
    if (swallow) { swallow = false; return; }
    const ws = wins.of(p);
    const w = ws.length ? nextWindow({ key: '', path: p.path, name: '', windows: ws }) : undefined;
    if (w) ipc.appActivate(w.id); else ipc.appLaunch(p.path, p.args ?? '', p.aumid ?? null).catch(() => {});
    close();
  }
  async function add() {
    const t = await ipc.appBrowse().catch(() => null);
    if (t && folder) save(addApps(folder, [{ path: t.path, args: t.args || undefined }]));
  }
  function menu(e: MouseEvent, p: Pin, i: number) {
    e.preventDefault();
    if (!folder) return;
    const f = folder;
    showMenu([
      { text: 'Open', action: () => launch(p) },
      { text: 'Show in folder', action: () => ipc.downloadsReveal(p.path).catch(() => {}) },
      '-',
      ...(i > 0 ? [{ text: 'Move left', action: () => save(moveApp(f, pinKey(p), i - 1)) }] : []),
      ...(i < f.apps.length - 1 ? [{ text: 'Move right', action: () => save(moveApp(f, pinKey(p), i + 1)) }] : []),
      { text: 'Take out of the folder', action: () => save(removeApp(f, pinKey(p))) },
    ]);
  }

  // ---- Drag: hold or move to pick an app up; it takes the place of the nearest cell, the
  // others make room. Released outside the grid, it leaves the folder. Esc puts it back. ----
  let grid = $state<HTMLElement>();
  let dragKey = $state<string | null>(null);
  let order = $state<Pin[] | null>(null);
  let outside = $state(false);
  let swallow = false;
  const shown = $derived(order ?? folder?.apps ?? []);
  function down(e: PointerEvent, p: Pin) {
    if (e.button !== 0 || !folder) return;
    const start = { x: e.clientX, y: e.clientY };
    let picked = false;
    const pick = () => { picked = true; dragKey = pinKey(p); order = [...folder!.apps]; };
    const timer = setTimeout(pick, 250);
    const move = (ev: PointerEvent) => {
      if (!picked && Math.hypot(ev.clientX - start.x, ev.clientY - start.y) > 6) { clearTimeout(timer); pick(); }
      if (!picked || !order || !grid) return;
      const g = grid.getBoundingClientRect();
      outside = ev.clientX < g.left - 24 || ev.clientX > g.right + 24 || ev.clientY < g.top - 24 || ev.clientY > g.bottom + 24;
      if (outside) return;
      const cells = [...grid.querySelectorAll<HTMLElement>('.app')].map((c) => { const r = c.getBoundingClientRect(); return Math.hypot(ev.clientX - (r.left + r.width / 2), ev.clientY - (r.top + r.height / 2)); });
      const to = cells.indexOf(Math.min(...cells));
      const from = order.findIndex((x) => pinKey(x) === dragKey);
      if (to >= 0 && to !== from) order = moveApp({ id, name: '', apps: order }, dragKey!, to).apps;
    };
    const finish = (keep: boolean) => {
      clearTimeout(timer);
      removeEventListener('pointermove', move);
      removeEventListener('pointerup', up);
      removeEventListener('keydown', esc);
      if (picked && keep && folder) {
        swallow = !outside; // released outside: no click follows to swallow
        save(outside ? removeApp(folder, dragKey!) : { ...folder, apps: order ?? folder.apps });
      }
      dragKey = null; order = null; outside = false;
    };
    const up = () => finish(true);
    const esc = (k: KeyboardEvent) => { if (k.key === 'Escape') finish(false); };
    addEventListener('pointermove', move);
    addEventListener('pointerup', up);
    addEventListener('keydown', esc);
  }

  // Programs and shortcuts dropped from Explorer anywhere on the panel.
  let dropping = $state(false);
  $effect(() => {
    const off = getCurrentWebviewWindow().onDragDropEvent((e) => {
      const d = e.payload;
      if (d.type === 'enter' || d.type === 'over') { dropping = true; return; }
      dropping = false;
      if (d.type === 'drop' && folder) {
        const apps = d.paths.filter(isApp).map((path) => ({ path }));
        if (apps.length) save(addApps(folder, apps));
      }
    });
    return () => { off.then((f) => f()).catch(() => {}); };
  });
</script>

{#if folder}
  <div class="folderp" class:dropping>
    <input class="name" value={folder.name} placeholder="Folder name (optional)" maxlength="32" aria-label="Folder name"
           onchange={(e) => save({ ...folder, name: e.currentTarget.value.trim() })}
           onkeydown={(e) => { if (e.key === 'Enter') e.currentTarget.blur(); }} />
    <div class="grid" bind:this={grid}>
      {#each shown as p, i (pinKey(p))}
        {@const src = iconOf(p)}
        {@const on = wins.of(p).length > 0}
        <button class="app" class:lifted={dragKey === pinKey(p)} class:leaving={dragKey === pinKey(p) && outside}
                title={dragKey === pinKey(p) && outside ? 'Release to take it out of the folder' : p.path}
                onpointerdown={(e) => down(e, p)} onclick={() => launch(p)} oncontextmenu={(e) => menu(e, p, i)}>
          {#if src}<img src={src} alt="" width="36" height="36" draggable="false" />{:else}<i class="ph">{pinName(p)[0]?.toUpperCase()}</i>{/if}
          <span class="nm ellipsis">{pinName(p)}</span>
          {#if on}<span class="run"></span>{/if}
        </button>
      {/each}
      <button class="app add" onclick={add} title="Add a program or shortcut"><span class="plus"><Plus size={18} /></span><span class="nm">Add app</span></button>
    </div>
    <p class="hint dim">{outside ? 'Release to take it out of the folder' : folder.apps.length ? 'Drag to reorder · drag out to remove · drop programs here to add' : 'Drop programs or shortcuts here, or use Add app'}</p>
  </div>
{:else}
  <p class="dim hint">This folder was deleted.</p>
{/if}

<style>
  .folderp { display: flex; flex-direction: column; gap: 10px; border-radius: var(--r-outer); transition: box-shadow 0.2s var(--ease); }
  .folderp.dropping { box-shadow: inset 0 0 0 2px var(--accent); }
  .name {
    height: 36px; padding: 0 10px; border-radius: var(--r-inner); border: 1px solid transparent; background: transparent;
    color: var(--fg); font: inherit; font-size: 16px; font-weight: 600; text-align: center;
    transition: background 0.2s var(--ease), border-color 0.2s var(--ease);
  }
  .name:hover { background: var(--pill); }
  .name:focus { outline: none; background: var(--bg-2, var(--pill)); border-color: var(--accent); }
  .name::placeholder { color: var(--fg-dim); font-weight: 500; }
  .grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 6px; }
  .app {
    position: relative; display: flex; flex-direction: column; align-items: center; gap: 4px; min-width: 0; padding: 8px 4px 6px;
    border-radius: var(--r-inner); touch-action: none;
    transition: background 0.2s var(--ease), transform 0.22s var(--spring), opacity 0.2s var(--ease);
    animation: pop 0.22s var(--ease) backwards;
  }
  .app:nth-child(2) { animation-delay: 30ms; } .app:nth-child(3) { animation-delay: 60ms; } .app:nth-child(4) { animation-delay: 90ms; }
  @keyframes pop { from { opacity: 0; transform: scale(0.9); } }
  .app:hover { background: var(--pill-hover); }
  .app:active { transform: scale(0.94); }
  .app:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
  .app.lifted { transform: scale(1.08); background: var(--pill-hover); box-shadow: 0 6px 18px rgba(0, 0, 0, 0.25); z-index: 1; }
  .app.leaving { opacity: 0.45; transform: scale(0.9); }
  .app img { width: 36px; height: 36px; object-fit: contain; }
  .ph { display: grid; place-items: center; width: 36px; height: 36px; border-radius: calc(10px * var(--rs)); font-style: normal; font-weight: 600;
        background: var(--accent); color: var(--on-accent); }
  .nm { max-width: 100%; font-size: 11px; text-align: center; }
  .run { position: absolute; top: 6px; right: 12px; width: 6px; height: 6px; border-radius: 50%; background: var(--accent); }
  .add { color: var(--fg-dim); }
  .plus { display: grid; place-items: center; width: 36px; height: 36px; border-radius: calc(10px * var(--rs)); border: 1.5px dashed var(--line); }
  .add:hover .plus { border-color: var(--accent); color: var(--accent); }
  .hint { margin: 0; font-size: 11px; text-align: center; }
  @media (prefers-reduced-motion: reduce) { .app { animation: none; transition: none; } }
</style>
