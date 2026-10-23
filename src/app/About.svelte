<script lang="ts">
  // Settings › About: which version this is, and what changed (from CHANGELOG.md).
  import { getVersion } from '@tauri-apps/api/app';
  import { ExternalLink, RefreshCw } from '@lucide/svelte';
  import md from '../../CHANGELOG.md?raw';
  import { ipc } from '../lib/ipc';
  import { boldParts, parseChangelog } from './changelog';

  let { onupdates }: { onupdates: () => void } = $props();
  let version = $state('');
  getVersion().then((v) => (version = v));
  const releases = parseChangelog(md);
  let showAll = $state(false);
</script>

<div class="group head">
  <div class="name">
    <b>GlassBar</b>
    <span class="ver">Version {version || '…'}</span>
  </div>
  <p class="hint m0">A glass taskbar for Windows 11 with widgets.</p>
  <div class="acts">
    <button class="btn plain" onclick={onupdates}><RefreshCw size={14} /> Check for updates</button>
    <button class="btn plain" onclick={() => ipc.appLaunch('https://github.com/msolehinunijaya/glassBar/releases', '')}><ExternalLink size={14} /> Release notes on GitHub</button>
  </div>
</div>

<h2>What's new</h2>
{#each showAll ? releases : releases.slice(0, 1) as r (r.version)}
  <section class="group rel">
    <h3>{r.version}{#if r.version === version}<span class="now">This version</span>{/if}{#if r.date}<span class="date">{r.date}</span>{/if}</h3>
    {#each r.groups as g (g.title)}
      {#if g.title}<h4>{g.title}</h4>{/if}
      <ul>
        {#each g.items as item, i (i)}
          <li>{#each boldParts(item) as [t, b], j (j)}{#if b}<b>{t}</b>{:else}{t}{/if}{/each}</li>
        {/each}
      </ul>
    {/each}
  </section>
{/each}
{#if releases.length > 1}
  <button class="btn plain" onclick={() => (showAll = !showAll)}>{showAll ? 'Show only the latest' : `Show earlier versions (${releases.length - 1})`}</button>
{/if}

<style>
  .head { padding: 14px 16px; gap: 6px; }
  .name { display: flex; align-items: baseline; gap: 10px; }
  .name b { font-size: 18px; font-weight: 600; }
  .ver { font-variant-numeric: tabular-nums; color: var(--fg-dim); }
  .m0 { margin: 0; }
  .acts { display: flex; flex-wrap: wrap; gap: 8px; padding: 4px 0 2px; }
  .btn.plain { background: var(--pill); color: var(--fg); }
  .rel { padding: 12px 16px 6px; gap: 2px; }
  h3 { display: flex; align-items: center; gap: 10px; margin: 0; font-size: 15px; font-weight: 600; font-variant-numeric: tabular-nums; }
  .now { font-size: 11px; font-weight: 600; padding: 1px 8px; border-radius: 999px; background: var(--accent); color: #fff; }
  .date { margin-left: auto; font-size: 12px; font-weight: 400; color: var(--fg-dim); }
  h4 { margin: 10px 0 2px; font-size: 13px; font-weight: 600; color: var(--fg-dim); }
  ul { margin: 0 0 6px; padding-left: 18px; line-height: 1.55; max-width: 72ch; }
  li { margin: 2px 0; }
  li b { font-weight: 600; }
</style>
