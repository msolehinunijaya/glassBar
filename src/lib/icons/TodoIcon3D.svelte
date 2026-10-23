<script lang="ts">
  // Checklist with depth, the Notes icon's sibling (same light, shadow and gloss) in
  // a fresh green: a raised card with two ticked rows and one open box.
  let { size = 20 }: { size?: number } = $props();
  const id = $props.id();
</script>

<svg class="i3d" width={size} height={size} viewBox="0 0 32 32" aria-hidden="true">
  <defs>
    <linearGradient id="{id}-body" x1="0" y1="0" x2="0.35" y2="1">
      <stop offset="0" stop-color="#7BF1A8" />
      <stop offset="0.5" stop-color="#34C759" />
      <stop offset="1" stop-color="#1A9E46" />
    </linearGradient>
    <linearGradient id="{id}-gloss" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#fff" stop-opacity="0.6" />
      <stop offset="1" stop-color="#fff" stop-opacity="0" />
    </linearGradient>
    <linearGradient id="{id}-box" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#ffffff" />
      <stop offset="1" stop-color="#E3F7EA" />
    </linearGradient>
    <filter id="{id}-sh" x="-30%" y="-30%" width="160%" height="170%">
      <feDropShadow dx="0" dy="1.6" stdDeviation="1.4" flood-color="#064d22" flood-opacity="0.4" />
    </filter>
  </defs>
  <g filter="url(#{id}-sh)">
    <rect x="4.5" y="4.5" width="23" height="23" rx="6" fill="url(#{id}-body)" />
    <path d="M10.5 4.5h11a6 6 0 0 1 6 6v1H4.5v-1a6 6 0 0 1 6-6z" fill="url(#{id}-gloss)" />
    <rect x="4.5" y="4.5" width="23" height="23" rx="6" fill="none" stroke="#0B6B2E" stroke-opacity="0.35" stroke-width="0.8" />
    <!-- rows: ticked, ticked, open -->
    {#each [9.5, 15.5, 21.5] as y, i (y)}
      <rect x="8.5" y={y - 2.4} width="4.8" height="4.8" rx="1.4" fill="url(#{id}-box)" />
      {#if i < 2}<path d="M9.6 {y}l1.2 1.2 2.2-2.4" fill="none" stroke="#1A9E46" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />{/if}
      <rect x="15.2" y={y - 0.9} width={i === 2 ? 6.5 : 9} height="1.8" rx="0.9" fill="#fff" fill-opacity={i < 2 ? 0.6 : 0.95} />
    {/each}
  </g>
</svg>

<style>
  .i3d { display: block; flex: none; overflow: visible; transition: transform 0.3s var(--spring); }
  :global(.pill:hover) .i3d { transform: translateY(-1px) rotate(3deg) scale(1.08); }
  :global(.pill:active) .i3d { transform: scale(0.9); }
</style>
