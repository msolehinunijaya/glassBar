<script lang="ts">
  // The magnifier window: draws the enlarged App Launcher icons the dock sends, nothing else.
  // Clicks pass through it to the dock (set up in magnify.rs).
  import { onMount } from 'svelte';
  import { events, type MagIcon } from '../lib/ipc';

  let icons = $state<MagIcon[]>([]);
  onMount(() => {
    const off = events.magnifyFrame((i) => (icons = i));
    return () => { off.then((f) => f()); };
  });
</script>

{#each icons as i, n (n)}
  <img src={i.src} alt="" draggable="false" style:left="{i.x}px" style:top="{i.y}px" style:width="{i.size}px" style:height="{i.size}px" />
{/each}

<style>
  :global(html[data-window='magnify']), :global(html[data-window='magnify'] body) { margin: 0; background: transparent; overflow: hidden; }
  img { position: fixed; display: block; transition: left 60ms linear, top 60ms linear, width 60ms linear, height 60ms linear; }
  :global(:root[data-perf='low']) img { transition: none; }
</style>
