<script lang="ts">
  // "What's new" after an update: this version's changes (from CHANGELOG.md), once. Opened by
  // the dock on the first start of a new version; Close, Esc or a click elsewhere closes it.
  import { Sparkles } from '@lucide/svelte';
  import { onMount } from 'svelte';
  import md from '../../CHANGELOG.md?raw';
  import { ipc } from '../lib/ipc';
  import { boldParts, parseChangelog } from './changelog';

  let { version, close }: { version: string; close: () => void } = $props();
  const rel = $derived(parseChangelog(md).find((r) => r.version === version));
  let closeBtn = $state<HTMLButtonElement>();
  onMount(() => { ipc.whatsNewSeen().catch(() => {}); closeBtn?.focus(); });
  const all = () => { close(); ipc.openSettings('About').catch(() => {}); };
</script>

<div class="wn">
  <header>
    <span class="spark"><Sparkles size={18} /></span>
    <span class="col">
      <b>What's new in GlassBar {version}</b>
      <span class="dim">{rel?.date ? `Updated · ${rel.date}` : 'Updated'}</span>
    </span>
  </header>

  <div class="body">
    {#if rel}
      {#each rel.groups as g (g.title)}
        {#if g.title}<h4>{g.title}</h4>{/if}
        <ul>
          {#each g.items as item, i (i)}
            <li>{#each boldParts(item) as [t, b], j (j)}{#if b}<b>{t}</b>{:else}{t}{/if}{/each}</li>
          {/each}
        </ul>
      {/each}
    {:else}
      <p class="dim">GlassBar was updated to {version}.</p>
    {/if}
  </div>

  <footer>
    <button class="link" onclick={all}>All changes</button>
    <button class="close" bind:this={closeBtn} onclick={close}>Close</button>
  </footer>
</div>

<style>
  .wn { display: flex; flex-direction: column; gap: var(--s-3); }
  header { display: flex; align-items: center; gap: 10px; }
  .spark { display: grid; place-items: center; width: 36px; height: 36px; border-radius: var(--r-inner); background: var(--accent); color: var(--on-accent); flex: none; }
  .col { display: flex; flex-direction: column; line-height: 1.3; }
  .col b { font-size: 15px; font-weight: 600; }
  .col .dim { font-size: 12px; }
  /* The changes scroll inside; the header and Close stay put. */
  .body { max-height: 360px; overflow: auto; padding-right: 4px; }
  h4 { margin: 4px 0 6px; font-size: 12px; font-weight: 600; color: var(--fg-dim); text-transform: uppercase; letter-spacing: 0.04em; }
  ul { margin: 0 0 8px; padding-left: 18px; display: flex; flex-direction: column; gap: 7px; }
  li { font-size: 12.5px; line-height: 1.45; color: var(--fg-dim); }
  li b { color: var(--fg); font-weight: 600; }
  footer { display: flex; align-items: center; justify-content: space-between; padding-top: var(--s-2); border-top: 1px solid var(--border); }
  .link { color: var(--accent); font-size: 12px; }
  .close { height: 36px; padding: 0 22px; border-radius: var(--r-inner); background: var(--accent); color: var(--on-accent); font-weight: 600; font-size: 13px; transition: transform 0.15s var(--ease); }
  .close:active { transform: scale(0.97); }
  .close:focus-visible, .link:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
</style>
