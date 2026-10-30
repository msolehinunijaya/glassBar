<script lang="ts">
  // Recycle Bin can with depth, sibling of the To-Do and Notes icons (same light, shadow and
  // gloss) in frosted glass grey. `full`: crumpled paper over the rim. `open`: the lid lifts
  // (files are being dragged over it).
  let { size = 20, full = false, open = false }: { size?: number; full?: boolean; open?: boolean } = $props();
  const id = $props.id();
</script>

<svg class="i3d" class:open width={size} height={size} viewBox="0 0 32 32" aria-hidden="true">
  <defs>
    <linearGradient id="{id}-body" x1="0" y1="0" x2="1" y2="0.2">
      <stop offset="0" stop-color="#B9C2CF" />
      <stop offset="0.45" stop-color="#E9EEF4" />
      <stop offset="1" stop-color="#8E99A8" />
    </linearGradient>
    <linearGradient id="{id}-lid" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#F4F7FA" />
      <stop offset="1" stop-color="#A3AEBC" />
    </linearGradient>
    <linearGradient id="{id}-paper" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#ffffff" />
      <stop offset="1" stop-color="#DCE3EA" />
    </linearGradient>
    <filter id="{id}-sh" x="-30%" y="-30%" width="160%" height="170%">
      <feDropShadow dx="0" dy="1.6" stdDeviation="1.4" flood-color="#1f2a38" flood-opacity="0.4" />
    </filter>
  </defs>
  <g filter="url(#{id}-sh)">
    {#if full}
      <!-- crumpled paper peeking over the rim -->
      <path d="M9.5 10.5c.4-2.6 2.6-3.8 4.4-3 1-1.8 3.6-2 4.8-.4 1.8-.9 3.9.3 3.9 2.6l-.2.8H9.5z" fill="url(#{id}-paper)" stroke="#9aa6b4" stroke-width="0.5" />
      <path d="M13 7.8l1.3 1.6M18.4 7.4l-.8 2" stroke="#9aa6b4" stroke-width="0.5" stroke-linecap="round" />
    {/if}
    <!-- can: slightly tapered, ribbed -->
    <path d="M7.5 11h17l-1.6 15.2a2.6 2.6 0 0 1-2.6 2.3h-8.6a2.6 2.6 0 0 1-2.6-2.3z" fill="url(#{id}-body)" />
    <path d="M7.5 11h17l-1.6 15.2a2.6 2.6 0 0 1-2.6 2.3h-8.6a2.6 2.6 0 0 1-2.6-2.3z" fill="none" stroke="#5b6675" stroke-opacity="0.45" stroke-width="0.8" />
    {#each [12.3, 16, 19.7] as x (x)}
      <path d="M{x} 13.5l{(x - 16) * 0.08} 12" stroke="#5b6675" stroke-opacity="0.35" stroke-width="1.1" stroke-linecap="round" />
    {/each}
    <!-- lid -->
    <g class="lid">
      <rect x="6" y="8.4" width="20" height="3.2" rx="1.4" fill="url(#{id}-lid)" stroke="#5b6675" stroke-opacity="0.45" stroke-width="0.7" />
      <rect x="13" y="6.6" width="6" height="2.2" rx="1" fill="url(#{id}-lid)" stroke="#5b6675" stroke-opacity="0.45" stroke-width="0.6" />
    </g>
  </g>
</svg>

<style>
  .i3d { display: block; flex: none; overflow: visible; transition: transform 0.3s var(--spring); }
  .lid { transform-box: fill-box; transform-origin: 90% 100%; transition: transform 0.28s var(--spring); }
  .open .lid { transform: rotate(14deg) translate(1px, -2px); }
  :global(.pill:hover) .i3d { transform: translateY(-1px) scale(1.06); }
  :global(.pill:active) .i3d { transform: scale(0.9); }
</style>
