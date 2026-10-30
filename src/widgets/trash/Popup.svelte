<script lang="ts">
  // Recycle Bin panel: size and count, Empty (asks first), the most recently deleted items
  // with Restore, and per drive. After files were dropped on the dock icon, a banner says
  // what happened with Undo (puts exactly those files back).
  import { Check, FolderOpen, RefreshCw, RotateCcw, Trash2, Undo2 } from '@lucide/svelte';
  import { fly } from 'svelte/transition';
  import PanelHeader from '../../lib/PanelHeader.svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { events, ipc, type BinItem, type RecycleInfo } from '../../lib/ipc';
  import FileIcon from '../downloads/FileIcon.svelte';
  import { ago, fmtSize } from '../downloads/format';
  import { binView, confirmText, freeShare } from '../sysmon/recycle';
  import type { PopupProps } from '../types';
  import type { TrashSettings } from './index';
  import { fromLabel } from './trash';

  let { instance, settings }: PopupProps<TrashSettings> = $props();
  const RECENT = 8;
  let info = $state<RecycleInfo | null>(null);
  let error = $state<string | null>(null);
  let items = $state<BinItem[] | null>(null);
  let itemsErr = $state<string | null>(null);
  let asking = $state<{ root: string | null; items: number; size: number } | null>(null);
  let busy = $state(false);
  let note = $state<string | null>(null);
  let restoring = $state<string | null>(null);
  let refreshing = $state(false);

  const loadInfo = () => ipc.recycleInfo().then((i) => { info = i; error = null; }).catch((e) => { error = String(e); info = null; });
  const loadItems = () => ipc.trashItems(RECENT).then((l) => { items = l; itemsErr = null; }).catch((e) => { itemsErr = String(e); items = []; });
  const load = () => Promise.all([loadInfo(), loadItems()]);
  $effect(() => {
    load();
    const off = events.changed('recycle', load);
    return () => { off.then((f) => f()); };
  });
  async function refresh() { refreshing = true; await load(); refreshing = false; }

  const v = $derived(binView(info, error));
  let cancelBtn = $state<HTMLButtonElement>();
  $effect(() => { if (asking) cancelBtn?.focus(); }); // default focus: Cancel

  async function empty() {
    const a = asking!;
    asking = null; busy = true; note = null;
    try { await ipc.recycleEmpty(a.root); note = `Freed ${fmtSize(a.size)}`; } catch (e) { error = String(e); }
    busy = false;
    await load();
  }
  async function restore(it: BinItem) {
    restoring = it.id;
    try { await ipc.trashRestore([it.id]); note = `Restored “${it.name}”`; items = (items ?? []).filter((x) => x.id !== it.id); }
    catch (e) { note = `Couldn't restore: ${e}`; }
    restoring = null;
  }

  // The drop banner shows for a minute after a drop.
  const drop = $derived(settings.lastDrop && Date.now() - settings.lastDrop.at < 60_000 ? settings.lastDrop : null);
  async function undo() {
    if (!drop) return;
    busy = true;
    try {
      const n = await ipc.trashRestore(drop.paths);
      note = `Put back ${n} item${n === 1 ? '' : 's'}`;
    } catch (e) { note = `Couldn't undo: ${e}`; }
    updateWidgetSettings(instance, { lastDrop: null });
    busy = false;
  }
</script>

<PanelHeader title="Recycle Bin">
  <button class="icon-btn" class:spin={refreshing} onclick={refresh} aria-label="Refresh" title="Refresh"><RefreshCw size={15} /></button>
  <button class="icon-btn" onclick={() => ipc.appLaunch('shell:RecycleBinFolder')} aria-label="Open Recycle Bin" title="Open in File Explorer"><FolderOpen size={16} /></button>
</PanelHeader>

{#if drop}
  <div class="drop" class:bad={!drop.ok} transition:fly={{ y: -4, duration: 180 }} role="status">
    <span class="ok">{#if drop.ok}<Check size={13} strokeWidth={3} />{:else}!{/if}</span>
    <span class="txt">{drop.text}</span>
    {#if drop.paths.length}<button class="b" disabled={busy} onclick={undo}><Undo2 size={13} /> Undo</button>{/if}
  </div>
{/if}

<div class="hero">
  <b class:dim={!info?.items}>{busy ? 'Emptying…' : v.title}</b>
  <span class="dim">{v.detail}{#if info?.items && info.drives.length === 1 && info.drives[0].free} · would free {freeShare(info.size, info.drives[0].free)}% of {info.drives[0].root.slice(0, 2)}{/if}</span>
</div>
{#if note}<p class="note small" role="status">{note}</p>{/if}

{#if asking}
  <div class="confirm" role="alertdialog" aria-label="Empty Recycle Bin">
    <p>{confirmText(asking.items, asking.size, asking.root?.slice(0, 2))}</p>
    <div class="cbtns">
      <button class="b" bind:this={cancelBtn} onclick={() => (asking = null)}>Cancel</button>
      <button class="b solid" onclick={empty}>Delete permanently</button>
    </div>
  </div>
{:else}
  <button class="emptyb" disabled={!v.canEmpty || busy} onclick={() => (asking = { root: null, items: info!.items, size: info!.size })}>
    <Trash2 size={14} /> Empty Recycle Bin
  </button>
{/if}

<div class="sec">
  <h4 class="dim">Recently deleted</h4>
  {#if items === null}
    {#each { length: 3 } as _, i (i)}<div class="row skel"><span class="sk ic"></span><span class="sk ln"></span></div>{/each}
  {:else if itemsErr}
    <p class="dim small">{itemsErr}</p>
  {:else}
    {#each items as it (it.id)}
      <div class="row" out:fly={{ x: 20, duration: 160 }}>
        <span class="icon"><FileIcon path={it.id} size={20} /></span>
        <span class="col">
          <span class="name ellipsis" title={it.path}>{it.name}</span>
          <span class="from ellipsis">{fromLabel(it.from) || 'Unknown folder'}</span>
        </span>
        <span class="when">{it.deletedAt ? ago(it.deletedAt) : ''}</span>
        <button class="rb" disabled={restoring === it.id} onclick={() => restore(it)} aria-label="Restore {it.name}" title="Restore to {it.from}">
          <RotateCcw size={14} />
        </button>
      </div>
    {:else}
      <p class="dim small empty">Nothing here. Files you delete show up here.</p>
    {/each}
  {/if}
</div>

{#if info && info.drives.length > 1}
  <div class="sec">
    <h4 class="dim">By drive</h4>
    {#each info.drives as d (d.root)}
      <div class="row drv">
        <b>{d.root.slice(0, 2)}</b>
        <span class="col dim small">{d.items ? `${fmtSize(d.size)} · ${d.items.toLocaleString()} item${d.items === 1 ? '' : 's'}` : 'Empty'}</span>
        <button class="mini" disabled={!d.items || busy} onclick={() => (asking = { root: d.root, items: d.items, size: d.size })}>Empty</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .hero { display: flex; flex-direction: column; gap: 2px; margin: 4px 2px 12px; }
  .hero b { font-size: 32px; font-weight: 600; line-height: 1.1; font-variant-numeric: tabular-nums; }
  .hero span { font-size: 12px; }
  .emptyb { display: flex; align-items: center; justify-content: center; gap: 6px; width: 100%; height: 36px; border-radius: var(--r-inner);
            border: 1px solid color-mix(in srgb, #ff453a 55%, transparent); color: #ff453a; font-size: 13px; font-weight: 500;
            transition: background 0.15s var(--ease), transform 0.15s var(--ease); }
  .emptyb:hover:not(:disabled) { background: color-mix(in srgb, #ff453a 12%, transparent); }
  .emptyb:active:not(:disabled) { transform: scale(0.98); }
  .emptyb:disabled { opacity: 0.4; }
  .emptyb:focus-visible, .rb:focus-visible, .b:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .confirm { padding: 12px; border-radius: var(--r-inner); background: color-mix(in srgb, #ff453a 14%, var(--pill)); }
  .confirm p { margin: 0 0 10px; font-size: 12.5px; line-height: 1.4; }
  .cbtns { display: flex; justify-content: flex-end; gap: 6px; }
  .b { display: inline-flex; align-items: center; gap: 5px; height: 30px; padding: 0 12px; border-radius: var(--r-inner); background: var(--pill); font-size: 12px; }
  .b:hover:not(:disabled) { background: var(--pill-hover); }
  .solid { background: #ff453a; color: #fff; }
  .solid:hover { background: #ff5b50 !important; }
  .sec { margin-top: var(--s-4); padding-top: var(--s-3); border-top: 1px solid var(--border); }
  h4 { margin: 0 2px 6px; font-size: 12px; font-weight: 500; }
  .row { display: flex; align-items: center; gap: 10px; min-height: 40px; padding: 0 var(--s-2); border-radius: var(--r-inner); }
  .row:hover { background: var(--bg-2); }
  .icon { display: grid; place-items: center; width: 22px; flex: none; }
  .col { flex: 1; min-width: 0; display: flex; flex-direction: column; line-height: 1.25; }
  .name { font-size: 13px; }
  .from { font-size: 11px; color: var(--fg-3); }
  .when { flex: none; font-size: 11px; color: var(--fg-dim); font-variant-numeric: tabular-nums; }
  /* Restore shows on hover (and keyboard focus) so the rows stay calm. */
  .rb { display: grid; place-items: center; width: 28px; height: 28px; flex: none; border-radius: var(--r-inner); opacity: 0; transition: opacity 0.15s, background 0.15s; }
  .row:hover .rb, .rb:focus-visible { opacity: 1; }
  .rb:hover { background: var(--pill-hover); color: var(--accent); }
  .rb:disabled { opacity: 0.4; }
  .drv b { width: 22px; font-size: 13px; }
  .mini { font-size: 11px; height: 26px; padding: 0 10px; border-radius: var(--r-inner); background: var(--pill); color: #ff453a; }
  .mini:disabled { opacity: 0.35; }
  .drop { display: flex; align-items: center; gap: 9px; margin: 0 0 10px; padding: 8px 8px 8px 10px; border-radius: var(--r-inner); background: color-mix(in srgb, #30d158 14%, var(--pill)); }
  .drop.bad { background: color-mix(in srgb, #ff9f0a 16%, var(--pill)); }
  .drop .ok { display: grid; place-items: center; width: 22px; height: 22px; flex: none; border-radius: 50%; background: #30d158; color: #fff; font-weight: 700; font-size: 12px; }
  .drop.bad .ok { background: #ff9f0a; }
  .drop .txt { flex: 1; font-size: 12px; line-height: 1.3; }
  .note { margin: -6px 2px 10px; color: var(--fg-dim); }
  .small { font-size: 12px; }
  .empty { padding: 8px 2px; margin: 0; }
  .skel .sk { display: block; border-radius: 6px; background: var(--bg-2); animation: shimmer 1.2s ease-in-out infinite; }
  .skel .ic { width: 20px; height: 20px; }
  .skel .ln { flex: 1; height: 12px; }
  @keyframes shimmer { 50% { opacity: 0.5; } }
  .spin :global(svg) { animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  @media (prefers-reduced-motion: reduce) { .skel .sk, .spin :global(svg) { animation: none; } }
</style>
