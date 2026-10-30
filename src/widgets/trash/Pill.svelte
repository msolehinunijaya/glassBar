<script lang="ts">
  // The Recycle Bin on the dock: empty or full can, its size (medium), files dropped on it go
  // to the Recycle Bin (never deleted outright). Right-click: open, empty (Windows asks),
  // restore the last deleted item. Updates when a bin changes anywhere (Rust watches it).
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import TrashIcon3D from '../../lib/icons/TrashIcon3D.svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { events, ipc, type RecycleInfo } from '../../lib/ipc';
  import { showMenu } from '../../lib/menu';
  import { fmtSize } from '../downloads/format';
  import { hits } from '../shelf/shelf';
  import type { PillProps } from '../types';
  import type { TrashSettings } from './index';
  import { binLook, dropSummary } from './trash';

  let { instance, size, settings, vertical }: PillProps<TrashSettings> = $props();
  let info = $state<RecycleInfo | null>(null);
  let error = $state<string | null>(null);
  let over = $state(false);
  let bounce = $state(false);
  let box = $state<HTMLElement>();

  const load = () => ipc.recycleInfo().then((i) => { info = i; error = null; }).catch((e) => { info = null; error = String(e); });
  $effect(() => {
    load();
    const off = events.changed('recycle', load);
    // A safety net only: Windows' change notifications normally keep this current.
    const t = setInterval(load, 10 * 60_000);
    return () => { clearInterval(t); off.then((f) => f()); };
  });

  const look = $derived(binLook(info));
  const tip = $derived(error ? `Recycle Bin · ${error}` : !info ? 'Recycle Bin' : info.items
    ? `Recycle Bin · ${info.items.toLocaleString()} item${info.items === 1 ? '' : 's'} · ${fmtSize(info.size)}` : 'Recycle Bin · empty');

  // Files dragged in from Explorer (Tauri reports the drop in physical px; see shelf.ts hits).
  $effect(() => {
    const off = getCurrentWebviewWindow().onDragDropEvent(async (e) => {
      const d = e.payload;
      if (d.type === 'leave') { over = false; return; }
      if (!box) return;
      const inside = hits(box.getBoundingClientRect(), d.position.x, d.position.y, devicePixelRatio);
      if (d.type === 'enter' || d.type === 'over') { over = inside; return; }
      over = false;
      if (!inside || !d.paths.length) return;
      bounce = true;
      setTimeout(() => (bounce = false), 450);
      const r = await ipc.trashRecycle(d.paths).catch((err) => ({ moved: 0, failed: [[d.paths[0], String(err)]] as [string, string][] }));
      // The panel shows what happened, with Undo (it reads `lastDrop`).
      const moved = d.paths.filter((p) => !r.failed.some(([f]) => f === p)).slice(0, 50);
      updateWidgetSettings(instance, { lastDrop: { at: Date.now(), paths: moved, text: dropSummary(r), ok: r.failed.length === 0 } });
      ipc.popupOpen(instance, box.getBoundingClientRect(), 380, 520, false);
    });
    return () => { off.then((f) => f()).catch(() => {}); };
  });

  async function menu(e: MouseEvent) {
    e.preventDefault(); e.stopPropagation();
    const last = info?.items ? await ipc.trashItems(1).catch(() => []) : [];
    await showMenu([
      { text: 'Open Recycle Bin', action: () => ipc.appLaunch('shell:RecycleBinFolder') },
      { text: 'Empty Recycle Bin…', enabled: !!info?.items, action: () => ipc.trashEmptyConfirm().catch(() => {}) },
      ...(last[0] ? ['-' as const, { text: `Restore “${last[0].name}”`, action: () => ipc.trashRestore([last[0].id]).catch(() => {}) }] : []),
    ]);
  }
  const open = (e: MouseEvent) => { e.stopPropagation(); ipc.appLaunch('shell:RecycleBinFolder'); };
</script>

{#if vertical}
  <div class="side bin" class:over class:bounce bind:this={box} title={tip} role="button" tabindex="-1" oncontextmenu={menu} ondblclick={open}>
    <TrashIcon3D size={22} full={look === 'full'} open={over} />
    {#if info?.items}<span class="side-c">{fmtSize(info.size)}</span>{/if}
  </div>
{:else}
  <div class="pill bin" class:over class:bounce class:medium={size !== 'small'} bind:this={box} title={tip} role="button" tabindex="-1" oncontextmenu={menu} ondblclick={open}>
    <TrashIcon3D size={22} full={look === 'full'} open={over} />
    {#if size !== 'small'}
      <span class="sz" class:dim={!info?.items}>{error ? 'N/A' : !info ? '…' : info.items ? fmtSize(info.size) : 'Empty'}</span>
    {/if}
  </div>
{/if}

<style>
  .bin { gap: 7px; transition: box-shadow 0.2s var(--ease), background 0.2s var(--ease); }
  .pill.bin:not(.medium) { padding: 0 8px; }
  .sz { font-size: 12px; font-weight: 500; font-variant-numeric: tabular-nums; white-space: nowrap; }
  /* A drop target: the lid lifts and the can glows with the accent. */
  .over { box-shadow: 0 0 0 2px var(--accent), 0 0 14px color-mix(in srgb, var(--accent) 45%, transparent); background: color-mix(in srgb, var(--accent) 16%, var(--pill)); }
  .bounce :global(.i3d) { animation: drop 0.45s var(--spring); }
  @keyframes drop { 30% { transform: translateY(2px) scale(0.92, 1.06); } 60% { transform: translateY(-2px) scale(1.04); } }
  @media (prefers-reduced-motion: reduce) { .bounce :global(.i3d) { animation: none; } }
</style>
