<script lang="ts">
  // A few files parked by the dock. Drag one in, click to open it, drag it out to take it off.
  //
  // The shelf holds paths only, in the widget's settings (like the App Launcher's pins), so it
  // survives a restart, travels with a config export, and never touches the files themselves.
  // Opening, revealing and the icons all reuse the Downloads widget's commands.
  import { Archive, Plus } from '@lucide/svelte';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { events, ipc } from '../../lib/ipc';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { showMenu } from '../../lib/menu';
  import FileIcon from '../downloads/FileIcon.svelte';
  import type { PillProps } from '../types';
  import type { ShelfSettings } from './index';
  import { MAX, add, extOf, hits, nameOf, remove, removeAll } from './shelf';

  let { instance, settings, vertical }: PillProps<ShelfSettings> = $props();
  let box = $state<HTMLElement | undefined>();
  /** A drag from outside is over the shelf: show where it would land. */
  let over = $state(false);
  let missing = $state<Record<string, boolean>>({});

  const files = $derived(settings.files ?? []);
  const save = (next: string[]) => updateWidgetSettings(instance, { files: next });

  // Which shelved files have been moved or deleted since. Never silently drop them: show them
  // dimmed so the user can see what happened and take them off themselves.
  $effect(() => {
    const list = files;
    if (!list.length) { missing = {}; return; }
    ipc.pathsExist(list).then((ok) => {
      missing = Object.fromEntries(list.map((p, i) => [p, !ok[i]]));
    }).catch(() => {});
  });

  // Files dragged in from Explorer. Tauri intercepts the drop, so the webview gets no HTML5
  // event: the position arrives in physical pixels and is hit-tested against this pill, which
  // is what keeps a drop meant for another widget out of the shelf.
  $effect(() => {
    const off = getCurrentWebview().onDragDropEvent((e) => {
      const d = e.payload;
      if (d.type === 'leave') { over = false; return; }
      if (!box) return;
      const inside = hits(box.getBoundingClientRect(), d.position.x, d.position.y, devicePixelRatio);
      if (d.type === 'enter' || d.type === 'over') { over = inside; return; }
      over = false;
      if (inside && d.paths.length) save(add(files, d.paths));
    });
    return () => { off.then((f) => f()).catch(() => {}); };
  });

  // A drag-out that really landed somewhere takes the file off the shelf. A cancelled drag
  // reports nothing, so a stray drag can't lose anything. The instance check matters with two
  // Shelf widgets: otherwise both would drop the file one of them handed over.
  $effect(() => {
    const off = events.shelfDropped((d) => { if (d.instance === instance) save(removeAll(files, d.paths)); });
    return () => { off.then((f) => f()).catch(() => {}); };
  });

  const pick = async () => {
    try {
      const picked = await ipc.shelfPick();
      if (picked.length) save(add(files, picked));
    } catch { /* cancelled */ }
  };

  // preventDefault cancels the HTML5 drag so Windows can run its own, but the pointer
  // sequence then still ends as a click on this element -- which would open the file you were
  // trying to drag out. The timestamp makes that deterministic instead of a race with
  // DoDragDrop's modal loop.
  let dragAt = 0;
  function startDrag(e: DragEvent, path: string) {
    if (missing[path]) { e.preventDefault(); return; }
    e.preventDefault();
    dragAt = Date.now();
    ipc.shelfDrag(instance, [path]).catch(() => {});
  }
  function open(e: Event, path: string) {
    e.stopPropagation();
    if (Date.now() - dragAt < 600) return; // that click was the end of a drag
    if (missing[path]) return;
    ipc.downloadsOpen(path).catch(() => {});
  }

  const context = (e: MouseEvent, path: string) => {
    e.preventDefault();
    e.stopPropagation();
    // Everything but "take it off" needs the file to still be there.
    const here = !missing[path];
    showMenu([
      { text: 'Open', enabled: here, action: () => ipc.downloadsOpen(path).catch(() => {}) },
      { text: 'Show in folder', enabled: here, action: () => ipc.downloadsReveal(path).catch(() => {}) },
      { text: 'Open with…', enabled: here, action: () => ipc.downloadsOpenWith(path).catch(() => {}) },
      '-',
      { text: 'Take off the shelf', action: () => save(remove(files, path)) },
      { text: 'Empty the shelf', action: () => save([]), enabled: files.length > 1 },
    ]);
  };

  const stop = (e: Event) => e.stopPropagation();
  const down = (e: PointerEvent) => e.stopPropagation(); // buttons must not start a widget drag
  const addFile = (e: Event) => { stop(e); pick(); };
</script>

{#if vertical}
  <div class="side-stack" bind:this={box} class:over>
    {#each files as f (f)}
      <div class="side shelf-side" class:gone={missing[f]} title={nameOf(f)}>
        <button type="button" class="grab" draggable={!missing[f]} disabled={missing[f]}
                aria-label={missing[f] ? `${nameOf(f)} is no longer there` : `Open ${nameOf(f)}`}
                ondragstart={(e) => startDrag(e, f)} onpointerdown={down}
                onclick={(e) => open(e, f)} oncontextmenu={(e) => context(e, f)}>
          <FileIcon path={f} size={20} />
        </button>
      </div>
    {/each}
    {#if !files.length}
      <div class="side" title="Drag files here to park them"><Archive size={18} class="dim" /></div>
    {/if}
  </div>
{:else}
  <div class="pill shelf" bind:this={box} class:over title={files.length ? `${files.length} of ${MAX} on the shelf` : 'Drag files here to park them'}>
    {#each files as f (f)}
      <!-- A file that has been moved or deleted stays visible but does nothing when clicked or
           dragged, rather than failing silently (DESIGN_RULES section 7: dim it, don't hide it). -->
      <button type="button" class="item" class:gone={missing[f]} draggable={!missing[f]} disabled={missing[f]}
              title={missing[f] ? `${nameOf(f)} — moved or deleted. Right-click to take it off the shelf.` : nameOf(f)}
              aria-label={missing[f] ? `${nameOf(f)} is no longer there` : `Open ${nameOf(f)}`}
              ondragstart={(e) => startDrag(e, f)} onpointerdown={down}
              onclick={(e) => open(e, f)} oncontextmenu={(e) => context(e, f)}>
        <FileIcon path={f} size={18} />
        {#if settings.showNames}<span class="nm ellipsis">{nameOf(f)}</span>{:else if extOf(f)}<span class="ext">{extOf(f)}</span>{/if}
      </button>
    {/each}

    {#if !files.length}
      <Archive size={16} class="dim" />
      <span class="hint dim">Drag files here</span>
    {/if}

    {#if files.length < MAX}
      <button class="addbtn" onpointerdown={down} onclick={addFile} aria-label="Add a file to the shelf" title="Add a file"><Plus size={13} /></button>
    {/if}
  </div>
{/if}

<style>
  .shelf { gap: 4px; padding: 0 5px; }
  .item {
    display: flex; align-items: center; gap: 4px; max-width: 120px; min-width: 0;
    padding: 2px 4px; border-radius: calc(7px * var(--rs)); cursor: pointer;
    transition: background 0.2s var(--ease), transform 0.25s var(--spring);
  }
  .item { cursor: pointer; }
  .item:hover { background: var(--pill-hover); }
  .item:active { transform: scale(0.92); }
  .item:focus-visible, .grab:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
  /* Moved or deleted since it was shelved: shown, dimmed, not hidden (DESIGN_RULES section 7). */
  .item.gone { opacity: 0.4; text-decoration: line-through; }
  .nm { font-size: 11px; }
  .ext { font-size: 9px; font-weight: 600; letter-spacing: 0.02em; color: var(--fg-dim); }
  .hint { font-size: 11px; }
  .addbtn {
    display: grid; place-items: center; width: 20px; height: 20px; flex: none;
    border-radius: 50%; color: var(--fg-dim);
    transition: background 0.2s var(--ease), color 0.2s var(--ease);
  }
  .addbtn:hover { background: var(--pill-hover); color: var(--fg); }

  /* A drag from outside is over the shelf. */
  .over { box-shadow: inset 0 0 0 2px var(--accent); }
  .shelf-side.gone { opacity: 0.4; }
  .side-stack.over { box-shadow: inset 0 0 0 2px var(--accent); border-radius: var(--r-outer); }
</style>
