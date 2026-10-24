<script lang="ts">
  // Recent downloads: filter, search, open (click), show in folder, copy path, drag out into
  // another app, right-click for Open with… / Rename / Move to Recycle Bin (never a permanent
  // delete). Footer: the folder's total size and a button to open it.
  import { Copy, FolderOpen, Search, Settings2 } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc, type DlItem, type DlState } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { showMenu } from '../../lib/menu';
  import type { PopupProps } from '../types';
  import type { DownloadsSettings } from './index';
  import FileIcon from './FileIcon.svelte';
  import { fmtSize, kindOf, when, type Kind } from './format';
  import Options from './Options.svelte';

  let { instance, settings }: PopupProps<DownloadsSettings> = $props();
  const st = live<DlState | null>('downloads', ipc.downloadsState, null);
  const filters: [Kind | 'all', string][] = [['all', 'All'], ['documents', 'Documents'], ['images', 'Images'], ['videos', 'Videos'], ['archives', 'Archives'], ['installers', 'Installers']];
  let filter = $state<Kind | 'all'>('all');
  let q = $state('');
  let gone = $state<Set<string>>(new Set()); // files found missing since they were listed
  let note = $state('');
  let renaming = $state<string | null>(null);
  let copied = $state<string | null>(null);
  let showOpts = $state(false);
  const rows = $derived((st.v?.items ?? [])
    .filter((i) => !gone.has(i.path) && (filter === 'all' || kindOf(i.name) === filter) && (!q.trim() || i.name.toLowerCase().includes(q.trim().toLowerCase())))
    .slice(0, settings.limit));

  async function act(i: DlItem, f: () => Promise<unknown>) {
    note = '';
    try { await f(); } catch (e) {
      const m = String(e);
      if (m.includes('no longer exists')) { gone = new Set([...gone, i.path]); note = `"${i.name}": file no longer exists`; } else note = m;
    }
  }
  async function copy(i: DlItem) { await navigator.clipboard.writeText(i.path); copied = i.path; setTimeout(() => (copied = null), 1200); }
  async function rename(i: DlItem, name: string) {
    renaming = null;
    if (name.trim() && name.trim() !== i.name) await act(i, () => ipc.downloadsRename(i.path, name));
  }
  async function menu(e: MouseEvent, i: DlItem) {
    e.preventDefault();
    await showMenu([
      { text: 'Open', action: () => act(i, () => ipc.downloadsOpen(i.path)) },
      { text: 'Open with…', action: () => act(i, () => ipc.downloadsOpenWith(i.path)) },
      { text: 'Show in folder', action: () => act(i, () => ipc.downloadsReveal(i.path)) },
      { text: 'Copy path', action: () => copy(i) },
      { text: 'Rename', action: () => (renaming = i.path) },
      { text: 'Move to Recycle Bin', action: () => act(i, () => ipc.downloadsRecycle(i.path)) },
    ]);
  }
  // Drag a row out (into a chat, an email…): hand over to Windows' own drag once the
  // pointer has moved a little with the button held.
  function down(e: PointerEvent, i: DlItem) {
    if (e.button !== 0 || renaming) return;
    const x = e.clientX, y = e.clientY;
    const move = (ev: PointerEvent) => {
      if (Math.hypot(ev.clientX - x, ev.clientY - y) < 6) return;
      off(); act(i, () => ipc.downloadsDrag(i.path));
    };
    const off = () => { removeEventListener('pointermove', move); removeEventListener('pointerup', off); };
    addEventListener('pointermove', move); addEventListener('pointerup', off);
  }
</script>

<div class="head">
  <h3>Downloads</h3>
  <button class="icon-btn" class:on={showOpts} onclick={() => (showOpts = !showOpts)} aria-label="Downloads options" title="Options"><Settings2 size={15} /></button>
</div>
{#if showOpts}<div class="opts"><Options {settings} save={(p) => updateWidgetSettings(instance, p)} /></div>{/if}

<label class="search"><Search size={13} /><input placeholder="Search by name…" bind:value={q} /></label>
<div class="filters">
  {#each filters as [k, label] (k)}<button class:sel={filter === k} onclick={() => (filter = k)}>{label}</button>{/each}
</div>
{#if st.v?.error}<p class="err">{st.v.error}</p>{/if}
{#if note}<p class="err">{note}</p>{/if}

<div class="list">
  {#each rows as i (i.path)}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="row" onpointerdown={(e) => down(e, i)} oncontextmenu={(e) => menu(e, i)} title="{i.path}&#10;Click to open · drag into another app · right-click for more">
      {#if renaming === i.path}
        <FileIcon path={i.path} size={24} />
        <!-- svelte-ignore a11y_autofocus -->
        <input class="rn" value={i.name} autofocus onkeydown={(e) => { if (e.key === 'Enter') rename(i, e.currentTarget.value); if (e.key === 'Escape') renaming = null; }} onblur={(e) => rename(i, e.currentTarget.value)} />
      {:else}
        <button class="main" onclick={() => act(i, () => ipc.downloadsOpen(i.path))}>
          <FileIcon path={i.path} size={24} />
          <span class="col"><span class="nm">{i.name}</span><span class="sub dim">{fmtSize(i.size)} · {when(i.at)}</span></span>
        </button>
        <span class="acts">
          <button onclick={() => act(i, () => ipc.downloadsReveal(i.path))} title="Show in folder" aria-label="Show in folder"><FolderOpen size={14} /></button>
          <button onclick={() => copy(i)} title="Copy path" aria-label="Copy path">{#if copied === i.path}✓{:else}<Copy size={13} />{/if}</button>
        </span>
      {/if}
    </div>
  {:else}
    <p class="dim empty">{q || filter !== 'all' ? 'Nothing matches.' : 'No downloads yet.'}</p>
  {/each}
</div>

<div class="foot">
  <span class="dim">{st.v ? `${fmtSize(st.v.totalSize)} in this folder` : ''}</span>
  <button class="open" onclick={() => ipc.downloadsOpenFolder()}><FolderOpen size={14} /> Open Downloads folder</button>
</div>

<style>
  .head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; }
  h3 { margin: 0; font-size: 15px; font-weight: 600; }
  .icon-btn.on { color: var(--accent); }
  .opts { margin-bottom: 10px; }
  .search { display: flex; align-items: center; gap: 6px; padding: 0 10px; border-radius: calc(10px * var(--rs)); background: var(--pill); }
  .search input { flex: 1; border: none; background: transparent; color: inherit; padding: 8px 0; font: inherit; font-size: 12px; outline: none; }
  .filters { display: flex; flex-wrap: wrap; gap: 4px; margin: 8px 0; }
  .filters button { padding: 4px 9px; border-radius: 999px; background: var(--pill); font-size: 11px; }
  .filters .sel { background: var(--accent); color: #fff; }
  .list { display: flex; flex-direction: column; gap: 3px; }
  .row { display: flex; align-items: center; gap: 4px; padding: 3px; border-radius: calc(10px * var(--rs)); transition: background 0.15s; }
  .row:hover { background: var(--pill-hover); }
  .main { flex: 1; min-width: 0; display: flex; align-items: center; gap: 10px; padding: 3px 4px; text-align: left; }
  .col { display: flex; flex-direction: column; min-width: 0; line-height: 1.25; }
  .nm { font-size: 12.5px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .sub { font-size: 10.5px; }
  .acts { display: flex; gap: 1px; opacity: 0; transition: opacity 0.15s; }
  .row:hover .acts { opacity: 1; }
  .acts button { display: grid; place-items: center; width: 26px; height: 26px; border-radius: calc(7px * var(--rs)); font-size: 12px; }
  .acts button:hover { background: var(--pill); }
  .rn { flex: 1; font-size: 12.5px; padding: 6px 8px; border-radius: calc(8px * var(--rs)); border: 1px solid var(--accent); background: var(--pill); color: inherit; outline: none; }
  .empty { text-align: center; padding: 26px 0; font-size: 12px; }
  .err { color: #ff9f0a; font-size: 11.5px; margin: 0 2px 6px; }
  .foot { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 10px; font-size: 11px; }
  .open { display: inline-flex; align-items: center; gap: 6px; padding: 6px 10px; border-radius: calc(10px * var(--rs)); background: var(--pill); font-size: 12px; }
  .open:hover { background: var(--pill-hover); }
</style>
