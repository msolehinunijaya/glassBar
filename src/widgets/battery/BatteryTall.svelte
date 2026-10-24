<script lang="ts">
  // Upright battery for a left/right dock tile: the fill rises from the bottom, in the same
  // colours as the flat battery (green charging/plugged, orange ≤ 30 %, red ≤ 15 %); a bolt
  // while charging.
  import type { Battery } from '../../lib/ipc';

  let { b }: { b: Battery } = $props();
  const plugged = $derived(b.charging || b.onAc);
  const color = $derived(plugged ? '#30d158' : b.percent <= 15 ? '#ff453a' : b.percent <= 30 ? '#ff9f0a' : '#30d158');
  // Body inner area: y 5..25 (20 high).
  const h = $derived((Math.max(4, b.percent) / 100) * 20);
</script>

<svg class="tall" class:low={!plugged && b.percent <= 15} viewBox="0 0 16 28" width="16" height="28" aria-hidden="true">
  <rect x="5.5" y="0.75" width="5" height="2.5" rx="1" fill="currentColor" opacity="0.7" />
  <rect x="1" y="3.25" width="14" height="24" rx="3.2" fill="none" stroke="currentColor" stroke-width="1.5" />
  <rect x="3" y={25.25 - h} width="10" height={h} rx="1.6" fill={color} />
  {#if b.charging}<path d="M9 9 5.2 16h2.6l-.9 5L10.8 14H8.2z" fill="#fff" stroke="rgba(0,0,0,.35)" stroke-width=".5" />{/if}
</svg>

<style>
  .tall { flex: none; display: block; }
  .tall rect:nth-child(3) { transition: y 0.8s var(--ease), height 0.8s var(--ease); }
  .low { animation: lowpulse 1.6s ease-in-out infinite; }
  @keyframes lowpulse { 50% { opacity: 0.55; } }
  @media (prefers-reduced-motion: reduce) { .low { animation: none; } .tall rect { transition: none; } }
</style>
