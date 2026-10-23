<script lang="ts">
  // Yellow sticky note with depth: warm gradient body, glossy top light, a lifted
  // folded corner with its own shadow, and soft ruled lines. Pure SVG, crisp at any size.
  let { size = 20 }: { size?: number } = $props();
  const id = $props.id();
</script>

<svg class="i3d" width={size} height={size} viewBox="0 0 32 32" aria-hidden="true">
  <defs>
    <linearGradient id="{id}-body" x1="0" y1="0" x2="0.35" y2="1">
      <stop offset="0" stop-color="#FFF08A" />
      <stop offset="0.55" stop-color="#FFD43B" />
      <stop offset="1" stop-color="#F5B300" />
    </linearGradient>
    <linearGradient id="{id}-fold" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#FFE58A" />
      <stop offset="1" stop-color="#E09A00" />
    </linearGradient>
    <linearGradient id="{id}-gloss" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#fff" stop-opacity="0.7" />
      <stop offset="1" stop-color="#fff" stop-opacity="0" />
    </linearGradient>
    <filter id="{id}-sh" x="-30%" y="-30%" width="160%" height="170%">
      <feDropShadow dx="0" dy="1.6" stdDeviation="1.4" flood-color="#7a4b00" flood-opacity="0.38" />
    </filter>
  </defs>
  <g filter="url(#{id}-sh)">
    <!-- body with the bottom-right corner cut for the fold -->
    <path d="M7 4.5h18a2.5 2.5 0 0 1 2.5 2.5v13.6L20.6 27.5H7A2.5 2.5 0 0 1 4.5 25V7A2.5 2.5 0 0 1 7 4.5z" fill="url(#{id}-body)" />
    <path d="M7 4.5h18a2.5 2.5 0 0 1 2.5 2.5v4.5H4.5V7A2.5 2.5 0 0 1 7 4.5z" fill="url(#{id}-gloss)" />
    <!-- ruled lines -->
    <g stroke="#B37400" stroke-opacity="0.45" stroke-width="1.5" stroke-linecap="round">
      <path d="M9 13.5h13" /><path d="M9 17.5h13" /><path d="M9 21.5h7.5" />
    </g>
    <!-- folded corner -->
    <path d="M20.6 27.5v-4.4a2.5 2.5 0 0 1 2.5-2.5h4.4z" fill="url(#{id}-fold)" />
    <path d="M20.6 27.5v-4.4a2.5 2.5 0 0 1 2.5-2.5h4.4" fill="none" stroke="#fff" stroke-opacity="0.55" stroke-width="0.6" />
    <path d="M7 4.5h18a2.5 2.5 0 0 1 2.5 2.5v13.6L20.6 27.5H7A2.5 2.5 0 0 1 4.5 25V7A2.5 2.5 0 0 1 7 4.5z" fill="none" stroke="#A86A00" stroke-opacity="0.35" stroke-width="0.8" />
  </g>
</svg>

<style>
  .i3d { display: block; flex: none; overflow: visible; transition: transform 0.3s var(--spring), filter 0.2s; }
  :global(.pill:hover) .i3d { transform: translateY(-1px) rotate(-4deg) scale(1.08); }
  :global(.pill:active) .i3d { transform: scale(0.9); }
</style>
