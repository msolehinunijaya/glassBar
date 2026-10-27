<script lang="ts">
  // The shelf, in full: every file with its name and where it came from, and the same actions
  // the dock's right-click menu offers.
  import { Archive, FolderOpen, Plus, X } from '@lucide/svelte';
  import PanelHeader from '../../lib/PanelHeader.svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc } from '../../lib/ipc';
  import FileIcon from '../downloads/FileIcon.svelte';
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { PopupProps } from '../types';
  import type { ShelfSettings } from './index';
  import { MAX, add, nameOf, remove } from './shelf';

  let { instance, settings }: PopupProps<ShelfSettings> = $props();
  const files = $derived(settings.files ?? []);
  const save = (next: string[]) => updateWidgetSettings(instance, { files: next });
  let gone = $state<Record<string, boolean>>({});

  $effect(() => {
    const list = files;
    if (!list.length) { gone = {}; return; }
    ipc.pathsExist(list).then((ok) => { gone = Object.fromEntries(list.map((p, i) => [p, !ok[i]])); }).catch(() => {});
  });

  /** The folder a file sits in, for the second line. */
  const folderOf = (p: string) => p.slice(0, Math.max(p.lastIndexOf('\\'), p.lastIndexOf('/'))) || p;

  const pick = async () => {
    try {
      const picked = await ipc.shelfPick();
      if (picked.length) save(add(files, picked));
    } catch { /* cancelled */ }
  };

  function startDrag(e: DragEvent, path: string) {
    e.preventDefault(); // hand the drag to Windows so the file can land in other apps
    ipc.shelfDrag([path]);
  }
</script>

<PanelHeader title="File Shelf">
  {#if files.length < MAX}
    <button class="icon-btn" onclick={pick} aria-label="Add a file" title="Add a file"><Plus size={16} /></button>
  {/if}
</PanelHeader>

{#if files.length}
  <ul class="list">
    {#each files as f (f)}
      <li class="row" class:gone={gone[f]}>
        <button type="button" class="grab" draggable="true" ondragstart={(e) => startDrag(e, f)} onclick={() => ipc.downloadsOpen(f)}
                title="Click to open · drag out to take it off the shelf">
          <FileIcon path={f} size={28} />
          <span class="meta">
            <span class="nm ellipsis">{nameOf(f)}</span>
            <span class="where ellipsis dim">{gone[f] ? 'Moved or deleted' : folderOf(f)}</span>
          </span>
        </button>
        <span class="acts">
          <button class="icon-btn" onclick={() => ipc.downloadsReveal(f)} aria-label="Show in folder" title="Show in folder"><FolderOpen size={15} /></button>
          <button class="icon-btn" onclick={() => save(remove(files, f))} aria-label="Take off the shelf" title="Take off the shelf"><X size={15} /></button>
        </span>
      </li>
    {/each}
  </ul>
  <p class="foot dim">{files.length} of {MAX}. Drag a file out into Explorer, a chat or an email and it comes off the shelf — the file itself stays where it is.</p>
{:else}
  <div class="empty dim">
    <Archive size={36} />
    <p>The shelf is empty.<br />Drag up to {MAX} files onto it to carry them around.</p>
    <button class="btn" onclick={pick}>Add a file</button>
  </div>
{/if}

<OptionsPopover title="Shelf" values={settings} save={(p) => updateWidgetSettings(instance, p)} options={[
  { type: 'toggle', key: 'showNames', label: 'Show file names on the dock', hint: 'Off: just the icon and its type' },
]} />

<style>
  .list { display: flex; flex-direction: column; gap: 2px; margin: 4px 0 0; padding: 0; list-style: none; }
  .row { display: flex; align-items: center; gap: 8px; height: 40px; padding: 0 4px 0 8px; border-radius: var(--r-inner); }
  .row:hover { background: var(--bg-2, var(--pill)); }
  .row.gone .nm { text-decoration: line-through; }
  .row.gone { opacity: 0.55; }
  .grab { display: flex; align-items: center; gap: 8px; flex: 1; min-width: 0; cursor: pointer; text-align: left; border-radius: var(--r-inner); }
  .grab:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .meta { display: flex; flex-direction: column; min-width: 0; line-height: 1.25; }
  .nm { font-size: 12px; font-weight: 500; }
  .where { font-size: 10.5px; }
  .acts { display: flex; gap: 2px; opacity: 0; transition: opacity 0.2s var(--ease); }
  .row:hover .acts, .row:focus-within .acts { opacity: 1; }
  .foot { font-size: 11px; line-height: 1.4; margin: 10px 2px 0; }
  .empty { text-align: center; padding: 34px 10px 24px; }
  .empty p { margin: 10px 0 14px; font-size: 12px; line-height: 1.5; }
</style>
