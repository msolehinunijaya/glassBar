<script lang="ts">
  // One capture icon in the chosen style: colour tile (white symbol on its colour), simple
  // (plain symbol in the text colour) or outline (the symbol drawn in its colour).
  import type { Component } from 'svelte';
  import type { CaptureTab, IconStyle } from './index';
  import { TINT, tile } from './look';

  let { k, icon: Icon, look = 'tile', size = 22 }: { k: CaptureTab; icon: Component<any>; look?: IconStyle; size?: number } = $props();
</script>

{#if look === 'tile'}
  <span class="ctile" style:width="{size}px" style:height="{size}px" style:background={tile(k)}><Icon size={Math.round(size * 0.6)} strokeWidth={2.3} /></span>
{:else if look === 'outline'}
  <span class="glyph outline" style:--c1={TINT[k][0]} style:--c2={TINT[k][1]}><Icon size={Math.round(size * 0.78)} strokeWidth={2.1} /></span>
{:else}
  <span class="glyph"><Icon size={Math.round(size * 0.72)} /></span>
{/if}

<style>
  /* (not ".tile": that name is taken by the shared Quick Settings tile style) */
  .ctile { display: grid; place-items: center; border-radius: calc(7px * var(--rs)); color: #fff; flex: none;
          box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.35), 0 1px 3px rgba(0, 0, 0, 0.25); }
  .glyph { display: grid; place-items: center; flex: none; }
  /* Outline: the deeper shade on light themes, the brighter one on dark (readable on both). */
  .outline { color: var(--c2); }
  :global([data-theme='dark']) .outline { color: var(--c1); }
</style>
