<script lang="ts">
  // The app that is playing, as a small icon on the album art's corner. Click: bring that
  // app to the front (or start it). Its name is in the tooltip.
  import type { Source } from './media.svelte';
  import { openSource } from './media.svelte';

  let { s, size = 14 }: { s: Source; size?: number } = $props();
</script>

<button class="badge" style:--b="{size}px" title="Playing in {s.name} · click to open it" aria-label="Open {s.name}"
        onpointerdown={(e) => e.stopPropagation()} onclick={(e) => { e.stopPropagation(); openSource(); }}>
  {#if s.icon}<img src={s.icon} alt="" draggable="false" />{:else}<span>{s.name[0]?.toUpperCase()}</span>{/if}
</button>

<style>
  .badge {
    position: absolute; right: -4px; bottom: -4px; z-index: 2; display: grid; place-items: center;
    width: calc(var(--b) + 4px); height: calc(var(--b) + 4px); padding: 2px; border-radius: 50%;
    background: var(--bg-1, #1c1c1e); box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35), 0 0 0 1px var(--line-soft);
    transition: transform 0.2s var(--spring, ease);
  }
  .badge:hover { transform: scale(1.25); }
  .badge:active { transform: scale(0.9); }
  .badge:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
  img { width: var(--b); height: var(--b); object-fit: contain; }
  span { font-size: 9px; font-weight: 700; }
</style>
