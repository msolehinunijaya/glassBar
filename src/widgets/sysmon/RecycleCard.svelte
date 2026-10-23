<script lang="ts">
  // Recycle Bin card: size and items (per drive too), Open, and Empty after a confirm.
  // Refreshes when the panel opens, every 60 s while it is open, and after an Empty or a
  // "Move to Recycle Bin" from the Downloads widget.
  import { FolderOpen, Trash2 } from '@lucide/svelte';
  import { events, ipc, type RecycleInfo } from '../../lib/ipc';
  import { fmtSize } from '../downloads/format';
  import { binView, confirmText, freeShare } from './recycle';

  let info = $state<RecycleInfo | null>(null);
  let error = $state<string | null>(null);
  let perDrive = $state(false);
  let asking = $state<{ root: string | null; items: number; size: number } | null>(null);
  let busy = $state(false);
  let freed = $state<string | null>(null);
  const load = () => ipc.recycleInfo().then((i) => { info = i; error = null; }).catch((e) => { error = String(e); info = null; });
  $effect(() => {
    load();
    const t = setInterval(load, 60_000);
    const off = events.changed('recycle', load);
    return () => { clearInterval(t); off.then((f) => f()); };
  });
  const v = $derived(binView(info, error));
  let cancelBtn = $state<HTMLButtonElement>();
  $effect(() => { if (asking) cancelBtn?.focus(); }); // default focus: Cancel

  async function empty() {
    const a = asking!;
    asking = null; busy = true; freed = null;
    try { await ipc.recycleEmpty(a.root); freed = `Freed ${fmtSize(a.size)}`; } catch (e) { error = String(e); }
    busy = false;
    await load();
  }
</script>

<h4 class="dim h"><Trash2 size={13} /> Recycle Bin</h4>
<div class="card bin">
  <div class="top">
    <div class="col">
      <b>{busy ? 'Emptying…' : v.title}</b>
      <span class="dim small">{freed ?? v.detail}</span>
    </div>
    <div class="btns">
      <button class="b" onclick={() => ipc.appLaunch('shell:RecycleBinFolder')} title="Open Recycle Bin"><FolderOpen size={14} /> Open</button>
      <button class="b danger" disabled={!v.canEmpty || busy} onclick={() => (asking = { root: null, items: info!.items, size: info!.size })}><Trash2 size={14} /> Empty</button>
    </div>
  </div>
  {#if info && info.drives.length > 1}
    <button class="link" onclick={() => (perDrive = !perDrive)}>{perDrive ? 'Hide' : 'Show'} per drive</button>
  {/if}
  {#if info && (perDrive || info.drives.length === 1)}
    {#each info.drives as d (d.root)}
      <div class="drv small">
        <span><b>{d.root.slice(0, 2)}</b> {d.items ? `${fmtSize(d.size)} · ${d.items} item${d.items === 1 ? '' : 's'}` : 'empty'}
          {#if d.items && d.free}<span class="dim"> · would free {freeShare(d.size, d.free)}% of {d.root.slice(0, 2)}</span>{/if}</span>
        {#if info.drives.length > 1}<button class="mini" disabled={!d.items || busy} onclick={() => (asking = { root: d.root, items: d.items, size: d.size })}>Empty</button>{/if}
      </div>
    {/each}
  {/if}
  {#if asking}
    <div class="confirm" role="alertdialog" aria-label="Empty Recycle Bin">
      <p>{confirmText(asking.items, asking.size, asking.root?.slice(0, 2))}</p>
      <div class="cbtns">
        <button class="b" bind:this={cancelBtn} onclick={() => (asking = null)}>Cancel</button>
        <button class="b danger solid" onclick={empty}>Delete permanently</button>
      </div>
    </div>
  {/if}
</div>

<style>
  h4 { margin: 14px 2px 6px; font-size: 12px; font-weight: 500; }
  .h { display: inline-flex; align-items: center; gap: 5px; }
  .bin { padding: 10px 12px; display: flex; flex-direction: column; gap: 6px; }
  .top { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .col { display: flex; flex-direction: column; }
  .col b { font-size: 18px; font-variant-numeric: tabular-nums; }
  .btns { display: flex; gap: 4px; }
  .b { display: inline-flex; align-items: center; gap: 5px; padding: 6px 10px; border-radius: 9px; background: var(--pill); font-size: 12px; }
  .b:hover:not(:disabled) { background: var(--pill-hover); }
  .b:disabled { opacity: 0.4; }
  .danger { color: #ff453a; }
  .solid { background: #ff453a; color: #fff; }
  .solid:hover { background: #ff5b50 !important; }
  .link { align-self: flex-start; color: var(--accent); font-size: 11.5px; }
  .drv { display: flex; align-items: center; justify-content: space-between; gap: 6px; }
  .mini { font-size: 11px; padding: 3px 8px; border-radius: 7px; background: var(--pill); color: #ff453a; }
  .mini:disabled { opacity: 0.35; }
  .small { font-size: 11.5px; }
  .confirm { margin-top: 4px; padding: 10px; border-radius: 10px; background: color-mix(in srgb, #ff453a 14%, var(--pill)); }
  .confirm p { margin: 0 0 8px; font-size: 12px; line-height: 1.4; }
  .cbtns { display: flex; justify-content: flex-end; gap: 6px; }
</style>
