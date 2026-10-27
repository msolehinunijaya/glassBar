<script lang="ts">
  // A water drop filled to `fill` (0..1).
  // - When the level rises (a drink was logged), the water sloshes and a droplet falls in.
  // - `fromEmpty`: on first show the level fills up from the bottom (the panel opening).
  // - `live`: the surface keeps gently moving (the open panel). Off on the dock, where an
  //   endless animation would cost CPU all day; there it only moves for a moment after a drink.
  import { onMount, untrack } from 'svelte';

  let { fill, size = 18, fromEmpty = false, live = false }: { fill: number; size?: number; fromEmpty?: boolean; live?: boolean } = $props();
  const id = `drop${Math.random().toString(36).slice(2, 8)}`;
  const clamp = (v: number) => Math.max(0, Math.min(1, v));

  // The starting level (deliberately the first value only; the effect below follows changes).
  let shown = $state(untrack(() => (fromEmpty ? 0 : clamp(fill))));
  let slosh = $state(false);
  let timer: ReturnType<typeof setTimeout>;
  onMount(() => {
    // Next frame, so the transition runs from empty.
    if (untrack(() => fromEmpty)) requestAnimationFrame(() => requestAnimationFrame(() => (shown = clamp(fill))));
    return () => clearTimeout(timer);
  });
  let last = untrack(() => clamp(fill));
  $effect(() => {
    const f = clamp(fill);
    if (f > last + 0.001) {
      slosh = false;
      requestAnimationFrame(() => (slosh = true)); // restart the slosh even mid-slosh
      clearTimeout(timer);
      timer = setTimeout(() => (slosh = false), 1600);
    }
    last = f;
    if (!fromEmpty || shown !== 0 || f === 0) shown = f;
  });
  const level = $derived(24 - shown * 20); // y of the surface in a 24-tall box
</script>

<svg class="drop" class:slosh class:live width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
  <defs>
    <clipPath id={id}><path d="M12 2.5c3.2 4.2 6.5 8 6.5 11.7a6.5 6.5 0 0 1-13 0C5.5 10.5 8.8 6.7 12 2.5Z" /></clipPath>
    <linearGradient id="{id}g" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#7fe0f0" /><stop offset="1" stop-color="#1fa8c8" />
    </linearGradient>
  </defs>
  <g clip-path="url(#{id})">
    <rect x="0" y="0" width="24" height="24" fill="var(--track)" />
    <g class="level" style:transform="translateY({level}px)">
      <path class="wave back" d="M-24 0.6 Q-18 -1 -12 0.6 T0 0.6 T12 0.6 T24 0.6 T36 0.6 V30 H-24 Z" fill="#40c8e0" opacity="0.45" />
      <path class="wave" d="M-24 0 Q-18 -1.6 -12 0 T0 0 T12 0 T24 0 T36 0 V30 H-24 Z" fill="url(#{id}g)" />
    </g>
  </g>
  <path d="M12 2.5c3.2 4.2 6.5 8 6.5 11.7a6.5 6.5 0 0 1-13 0C5.5 10.5 8.8 6.7 12 2.5Z" fill="none" stroke="#40c8e0" stroke-width="1.4" />
  <!-- The droplet that falls in when a drink is logged. -->
  <circle class="fall" cx="12" cy="-2" r="1.6" fill="#40c8e0" />
</svg>

<style>
  .drop { flex: none; overflow: visible; }
  .level { transition: transform 0.9s cubic-bezier(0.22, 1, 0.36, 1); }
  .fall { opacity: 0; }
  /* After a drink: the surface rolls for a moment and a droplet drops in. */
  .slosh .wave { animation: roll 0.8s ease-in-out 2; }
  .slosh .wave.back { animation-direction: reverse; }
  .slosh .fall { animation: fall 0.5s cubic-bezier(0.55, 0, 1, 0.45); }
  /* The open panel: the surface keeps moving gently. */
  .live .wave { animation: roll 3.2s linear infinite; }
  .live .wave.back { animation-duration: 4.4s; animation-direction: reverse; }
  @keyframes roll { to { transform: translateX(12px); } }
  @keyframes fall { 0% { opacity: 1; transform: translateY(0); } 85% { opacity: 1; } 100% { opacity: 0; transform: translateY(14px); } }
  @media (prefers-reduced-motion: reduce) { .level { transition: none; } .wave, .fall { animation: none !important; } }
  :global(:root[data-motion='reduced']) .wave, :global(:root[data-motion='reduced']) .fall { animation: none !important; }
</style>
