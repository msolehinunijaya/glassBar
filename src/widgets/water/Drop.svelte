<script lang="ts">
  // A water drop filled to `fill` (0..1), with a still wave on top. The level rises with a
  // transition when a drink is logged; nothing moves while idle (no endless animation).
  let { fill, size = 18 }: { fill: number; size?: number } = $props();
  const id = `drop${Math.random().toString(36).slice(2, 8)}`;
  const level = $derived(24 - Math.max(0, Math.min(1, fill)) * 20); // y of the surface in a 24-tall box
</script>

<svg class="drop" width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
  <defs>
    <clipPath id={id}><path d="M12 2.5c3.2 4.2 6.5 8 6.5 11.7a6.5 6.5 0 0 1-13 0C5.5 10.5 8.8 6.7 12 2.5Z" /></clipPath>
  </defs>
  <g clip-path="url(#{id})">
    <rect x="0" y="0" width="24" height="24" fill="var(--track)" />
    <g class="level" style:transform="translateY({level}px)">
      <path class="wave" d="M-12 0 Q-6 -1.6 0 0 T12 0 T24 0 T36 0 V30 H-12 Z" fill="#40c8e0" />
    </g>
  </g>
  <path d="M12 2.5c3.2 4.2 6.5 8 6.5 11.7a6.5 6.5 0 0 1-13 0C5.5 10.5 8.8 6.7 12 2.5Z" fill="none" stroke="#40c8e0" stroke-width="1.4" />
</svg>

<style>
  .drop { flex: none; overflow: visible; }
  .level { transition: transform 0.6s var(--ease); }
  @media (prefers-reduced-motion: reduce) { .level { transition: none; } }
</style>
