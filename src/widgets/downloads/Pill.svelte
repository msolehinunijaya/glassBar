<script lang="ts">
  // The latest finished download. Click the file to open it; the folder button shows it in
  // File Explorer; the rest of the pill (and the list button) opens the panel. A new
  // download pulses the pill and (optionally) shows a "new" dot until hovered or opened.
  // Small: icon + folder · Medium: + name · Large: + size and time.
  import { FolderOpen, List } from '@lucide/svelte';
  import { events, ipc, type DlState } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { tickingNow } from '../../lib/now.svelte';
  import type { PillProps } from '../types';
  import type { DownloadsSettings } from './index';
  import FileIcon from './FileIcon.svelte';
  import { ago, cutMiddle, fmtSize } from './format';

  let { size, settings, vertical }: PillProps<DownloadsSettings> = $props();
  const st = live<DlState | null>('downloads', ipc.downloadsState, null);
  const latest = $derived(st.v?.items[0]);
  const t = tickingNow(() => size === 'large' && !!latest, 30_000);
  let fresh = $state(false);
  let pulse = $state(false);
  $effect(() => {
    const off = events.downloadsNew(() => { fresh = settings.newDot; pulse = true; setTimeout(() => (pulse = false), 1600); });
    return () => { off.then((f) => f()); };
  });
  let err = $state('');
  async function run(e: Event, f: () => Promise<unknown>) {
    e.stopPropagation(); fresh = false; err = '';
    try { await f(); } catch (x) { err = String(x); setTimeout(() => (err = ''), 2500); }
  }
  const down = (e: PointerEvent) => e.stopPropagation();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="pill dl" class:pulse class:vertical onpointerenter={() => (fresh = false)}
     title={err || (latest ? `${latest.name} · ${fmtSize(latest.size)} · ${ago(latest.at)}` : 'No downloads')}>
  {#if latest}
    <button class="file" onpointerdown={down} onclick={(e) => run(e, () => ipc.downloadsOpen(latest.path))} aria-label="Open {latest.name}">
      <span class="ic"><FileIcon path={latest.path} size={20} />{#if fresh}<i class="dot"></i>{/if}</span>
      {#if size !== 'small' && !vertical}
        <span class="col">
          <span class="nm">{err || cutMiddle(latest.name, size === 'large' ? 26 : 20)}</span>
          {#if size === 'large'}<span class="sub dim">{fmtSize(latest.size)} · {ago(latest.at, t.now)}</span>{/if}
        </span>
      {/if}
    </button>
  {:else if size !== 'small' && !vertical}
    <span class="dim empty">{st.v?.error ? 'Folder unavailable' : 'No downloads'}</span>
  {/if}
  <button class="act" onpointerdown={down} title={latest ? 'Show in folder' : 'Open the Downloads folder'} aria-label="Show in folder"
          onclick={(e) => run(e, () => (latest ? ipc.downloadsReveal(latest.path) : ipc.downloadsOpenFolder()))}><FolderOpen size={15} /></button>
  {#if size !== 'small' && !vertical}<span class="act" title="Recent downloads"><List size={14} /></span>{/if}
</div>

<style>
  .dl { gap: 2px; padding: 0 6px; }
  .dl.vertical { flex-direction: column; padding: 4px 0; }
  .file { display: flex; align-items: center; gap: 7px; min-width: 0; padding: 3px 6px 3px 3px; border-radius: 9px; transition: background 0.15s; }
  .file:hover { background: var(--pill-hover); }
  .ic { position: relative; display: grid; }
  .dot { position: absolute; top: -2px; right: -3px; width: 8px; height: 8px; border-radius: 50%; background: var(--accent); box-shadow: 0 0 0 2px var(--pill); }
  .col { display: flex; flex-direction: column; line-height: 1.15; min-width: 0; text-align: left; }
  .nm { font-size: 12px; font-weight: 500; white-space: nowrap; }
  .sub { font-size: 10.5px; white-space: nowrap; }
  .empty { font-size: 12px; padding: 0 6px; }
  /* Ghost icon buttons (not the global filled .btn). They keep their place but only show on
     hover or keyboard focus, so the resting pill is just the file. Small size: the folder
     button is the only action, so it stays visible. */
  .act { display: grid; place-items: center; width: 26px; height: 26px; border-radius: var(--r-inner); color: var(--fg-dim);
         opacity: 0; transition: background 0.15s, opacity 0.15s, color 0.15s; }
  .dl:hover .act, .dl:focus-within .act, .dl.vertical .act, .dl:not(:has(.col)) .act { opacity: 1; }
  .act:hover { background: var(--pill-hover); color: var(--fg); }
  .act:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; opacity: 1; }
  /* A new download: a short accent pulse. */
  .pulse { animation: dlpulse 0.8s ease-in-out 2; }
  @keyframes dlpulse { 50% { box-shadow: 0 0 0 2px var(--accent), 0 0 14px color-mix(in srgb, var(--accent) 60%, transparent); } }
</style>
