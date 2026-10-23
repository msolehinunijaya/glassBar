<script lang="ts">
  import { fade } from './look';
  // Screenshot: full screen, an area, a window, or a whole scrolling page. Every shot is
  // copied right away and kept in the history below for the chosen number of days.
  import { AppWindow, Copy, ExternalLink, FolderOpen, Monitor, ScrollText, SquareDashed, Trash2 } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc, type ShotItem } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { relTime } from '../../lib/time';
  import type { OcrSettings } from './index';

  let { instance, settings }: { instance: string; settings: OcrSettings } = $props();
  const shots = live<ShotItem[]>('shots', ipc.shotsList, []);
  let copied = $state<number | null>(null);
  let busy = $state('');

  // The panel closes first so it isn't in the picture.
  async function take(kind: 'full' | 'region' | 'window' | 'scroll') {
    await ipc.popupHide();
    setTimeout(async () => {
      if (kind === 'full') await ipc.shotFullscreen().catch(() => {});
      else await ipc.ocrSnapStart(kind).catch(() => {});
    }, 180);
  }
  async function copy(s: ShotItem) {
    busy = 'copy' + s.id;
    await ipc.shotsCopy(s.id).catch(() => {});
    busy = ''; copied = s.id;
    setTimeout(() => (copied = null), 1200);
  }
  const modes = [
    ['full', 'Full screen', Monitor], ['region', 'Area', SquareDashed],
    ['window', 'Window', AppWindow], ['scroll', 'Scrolling', ScrollText],
  ] as const;
</script>

<div class="grp act">
<div class="modes">
  {#each modes as [k, label, Icon] (k)}
    <button class="mode" onclick={() => take(k)} title={k === 'scroll' ? 'Pick a window (e.g. a browser): it scrolls to the end and the page is captured as one tall image' : label}>
      <Icon size={20} /><span>{label}</span>
    </button>
  {/each}
</div>
<p class="dim small">Copied to the clipboard automatically. Kept for
  <select value={String(settings.keepDays)} onchange={(e) => updateWidgetSettings(instance, { keepDays: +e.currentTarget.value })}>
    {#each [1, 3, 7, 14, 30] as d (d)}<option value={String(d)}>{d} day{d > 1 ? 's' : ''}</option>{/each}
  </select>, then deleted.</p>
</div>

<div class="grp hist">
<div class="head">
  <h4 class="dim">Recent</h4>
  <span>
    <button class="link" onclick={() => ipc.shotsOpen(null)}><FolderOpen size={12} /> Folder</button>
    {#if shots.v.length}<button class="link" onclick={() => ipc.shotsDelete(null)}>Clear</button>{/if}
  </span>
</div>
<div class="grid">
  {#each shots.v as s, n (s.id)}
    <div class="shot" style:--fade={fade(n)}>
      <button class="thumb" title="Open" onclick={() => ipc.shotsOpen(s.id)}><img src={s.thumb} alt="" /></button>
      <div class="meta dim">
        <span>{relTime(s.at)} · {s.w}×{s.h}</span>
        <span class="acts">
          <button onclick={() => copy(s)} aria-label="Copy" title="Copy">{#if copied === s.id}✓{:else}<Copy size={12} />{/if}</button>
          <button onclick={() => ipc.shotsOpen(s.id)} aria-label="Open" title="Open"><ExternalLink size={12} /></button>
          <button onclick={() => ipc.shotsDelete(s.id)} aria-label="Discard" title="Discard"><Trash2 size={12} /></button>
        </span>
      </div>
    </div>
  {:else}
    <p class="dim small empty">No screenshots yet.</p>
  {/each}
</div>
</div>

<style>
  .modes { display: grid; grid-template-columns: repeat(4, 1fr); gap: 6px; }
  .mode { display: flex; flex-direction: column; align-items: center; gap: 5px; padding: 11px 4px 9px; border-radius: 14px; background: var(--pill); font-size: 11px; font-weight: 500; transition: background 0.15s, transform 0.2s var(--spring); }
  .mode:hover { background: var(--pill-hover); }
  .mode:active { transform: scale(0.95); }
  .small { font-size: 11px; margin: 8px 2px 0; }
  select { font-size: 11px; padding: 1px 4px; border-radius: 6px; background: var(--pill); color: inherit; border: none; }
  .head { display: flex; justify-content: space-between; align-items: center; margin: 12px 2px 6px; }
  h4 { margin: 0; font-size: 12px; font-weight: 500; }
  .link { display: inline-flex; align-items: center; gap: 3px; color: var(--accent); font-size: 12px; margin-left: 8px; }
  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .shot { border-radius: 12px; background: var(--pill); overflow: hidden; opacity: var(--fade, 1); transition: opacity 0.15s; }
  .shot:hover { opacity: 1; }
  .thumb { display: block; width: 100%; height: 86px; background: #0002; }
  .thumb img { width: 100%; height: 100%; object-fit: cover; object-position: top; display: block; }
  .meta { display: flex; align-items: center; justify-content: space-between; padding: 4px 6px; font-size: 10px; }
  .acts { display: flex; gap: 2px; }
  .acts button { display: grid; place-items: center; width: 20px; height: 20px; border-radius: 6px; color: var(--fg); opacity: 0.75; }
  .acts button:hover { opacity: 1; background: var(--pill-hover); }
  .empty { grid-column: 1 / -1; text-align: center; padding: 24px 0; }
</style>
