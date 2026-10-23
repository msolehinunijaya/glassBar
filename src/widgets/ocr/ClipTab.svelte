<script lang="ts">
  import { fade } from './look';
  // Clipboard history: copied text and images, newest first. Click to copy again.
  // Things apps mark as private (password managers) are never recorded.
  import { Copy, Search, Trash2 } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { events, ipc, type ClipItem } from '../../lib/ipc';
  import { relTime } from '../../lib/time';
  import type { OcrSettings } from './index';

  let { instance, settings }: { instance: string; settings: OcrSettings } = $props();
  let q = $state('');
  let items = $state<ClipItem[]>([]);
  let copied = $state<number | null>(null);
  const load = () => ipc.clipList(q || null).then((l) => (items = l)).catch(() => {});
  $effect(() => { q; load(); });
  $effect(() => { const off = events.changed('clip', load); return () => { off.then((f) => f()); }; });

  async function copy(c: ClipItem) {
    await ipc.clipCopy(c.id).catch(() => {});
    copied = c.id;
    setTimeout(() => (copied = null), 1200);
  }
</script>

<div class="grp act">
<div class="top">
  <label class="search"><Search size={13} /><input placeholder="Search copied text…" bind:value={q} /></label>
  <label class="dim sw" title="Record what you copy">On
    <input type="checkbox" class="switch" checked={settings.clipHistory} onchange={(e) => updateWidgetSettings(instance, { clipHistory: e.currentTarget.checked })} />
  </label>
</div>
{#if !settings.clipHistory}
  <p class="dim small">Clipboard history is off. Nothing you copy is recorded.</p>
{/if}
</div>
<div class="grp hist">
<div class="head"><h4 class="dim">Recent copies</h4>{#if items.length}<button class="link" onclick={() => ipc.clipDelete(null)}>Clear</button>{/if}</div>
<div class="list">
  {#each items as c, n (c.id)}
    <div class="item" style:--fade={fade(n)}>
      <button class="body" onclick={() => copy(c)} title="Copy again">
        {#if c.kind === 'image' && c.thumb}<img src={c.thumb} alt="Copied image" />{:else}<span class="txt">{c.text}</span>{/if}
      </button>
      <div class="meta dim">
        <span>{relTime(c.at)}{c.kind === 'image' ? ` · ${c.w}×${c.h}` : ''}</span>
        <span class="acts">
          <button onclick={() => copy(c)} aria-label="Copy">{#if copied === c.id}✓{:else}<Copy size={12} />{/if}</button>
          <button onclick={() => ipc.clipDelete(c.id)} aria-label="Delete"><Trash2 size={12} /></button>
        </span>
      </div>
    </div>
  {:else}
    <p class="dim small empty">{q ? 'No match.' : 'Copy something and it shows up here.'}</p>
  {/each}
</div>
<p class="dim small">Keeps the last 200 copies. Passwords copied from password managers are skipped.</p>
</div>

<style>
  .top { display: flex; align-items: center; gap: 8px; }
  .search { flex: 1; display: flex; align-items: center; gap: 6px; padding: 0 10px; border-radius: 10px; background: var(--pill); }
  .search input { flex: 1; border: none; background: transparent; color: inherit; padding: 8px 0; font: inherit; font-size: 12px; outline: none; }
  .sw { display: flex; align-items: center; gap: 6px; font-size: 11px; }
  .head { display: flex; justify-content: space-between; align-items: center; margin: 10px 2px 6px; }
  h4 { margin: 0; font-size: 12px; font-weight: 500; }
  .link { color: var(--accent); font-size: 12px; }
  .list { display: flex; flex-direction: column; gap: 6px; }
  .item { border-radius: 12px; background: var(--pill); overflow: hidden; opacity: var(--fade, 1); transition: opacity 0.15s; }
  .item:hover { opacity: 1; }
  .body { display: block; width: 100%; text-align: left; padding: 8px 10px 4px; }
  .body:hover { background: var(--pill-hover); }
  .body img { max-width: 100%; max-height: 90px; border-radius: 6px; display: block; }
  .txt { display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; font-size: 12px; line-height: 1.4; white-space: pre-wrap; word-break: break-word; }
  .meta { display: flex; justify-content: space-between; align-items: center; padding: 2px 8px 5px 10px; font-size: 10px; }
  .acts { display: flex; gap: 2px; }
  .acts button { display: grid; place-items: center; width: 20px; height: 20px; border-radius: 6px; color: var(--fg); opacity: 0.75; }
  .acts button:hover { opacity: 1; background: var(--pill-hover); }
  .small { font-size: 11px; margin: 8px 2px 0; }
  .empty { text-align: center; padding: 24px 0; }
</style>
