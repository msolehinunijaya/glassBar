<script lang="ts">
  // Liquid filling its container from the bottom to `fill` (0..1), in `color`: the Water
  // widget's whole background and panel, the battery's "Fill" design. Sits behind the content.
  // - A drink logged (the level rises): the surface rolls for a moment.
  // - `fromEmpty`: on first show it fills up from the bottom (the panel opening).
  // - `live`: the surface keeps moving gently (the open panel only; on the dock an endless
  //   animation would cost CPU all day).
  // - `fixed`: fill the whole panel window edge to edge, past the panel's padding (--pad,
  //   set by app/Popup.svelte); the panel's content box is the frame it is measured from.
  import { onMount, untrack } from 'svelte';

  let { fill, color = '#40c8e0', fromEmpty = false, live = false, fixed = false }: { fill: number; color?: string; fromEmpty?: boolean; live?: boolean; fixed?: boolean } = $props();
  const clamp = (v: number) => Math.max(0, Math.min(1, v));

  // Deliberately the first values only; the effect below follows changes.
  let shown = $state(untrack(() => (fromEmpty ? 0 : clamp(fill))));
  let started = untrack(() => !fromEmpty);
  let slosh = $state(false);
  let timer: ReturnType<typeof setTimeout>;
  onMount(() => {
    if (!started) requestAnimationFrame(() => requestAnimationFrame(() => { started = true; shown = clamp(fill); }));
    return () => clearTimeout(timer);
  });
  let last = untrack(() => clamp(fill));
  $effect(() => {
    const f = clamp(fill);
    if (f > last + 0.001) {
      slosh = false;
      requestAnimationFrame(() => (slosh = true));
      clearTimeout(timer);
      timer = setTimeout(() => (slosh = false), 2000);
    }
    last = f;
    if (started) shown = f;
  });
</script>

<div class="tank" class:fixed class:live class:slosh class:empty={shown <= 0} style:--c={color} aria-hidden="true">
  <div class="water" style:height="{shown * 100}%">
    <svg class="wave back" viewBox="0 0 240 12" preserveAspectRatio="none"><path d="M0 6 Q15 0 30 6 T60 6 T90 6 T120 6 T150 6 T180 6 T210 6 T240 6 V12 H0 Z" /></svg>
    <svg class="wave" viewBox="0 0 240 12" preserveAspectRatio="none"><path d="M0 6 Q15 12 30 6 T60 6 T90 6 T120 6 T150 6 T180 6 T210 6 T240 6 V12 H0 Z" /></svg>
  </div>
</div>

<style>
  /* A light wash of the liquid's colour over the whole box, top to bottom, edge to edge; the
     deeper liquid below rises to the level. No plain space anywhere. */
  .tank { position: absolute; inset: 0; z-index: 0; overflow: hidden; border-radius: inherit; pointer-events: none;
          background: color-mix(in srgb, var(--c) 13%, transparent); transition: background 0.6s var(--ease); }
  .tank.fixed { inset: calc(-1 * var(--pad, 14px)); border-radius: var(--r-window, 16px); }
  .water {
    position: absolute; left: 0; right: 0; bottom: 0;
    background: linear-gradient(180deg, color-mix(in srgb, var(--c) 34%, transparent), color-mix(in srgb, var(--c) 52%, transparent));
    transition: height 1.1s cubic-bezier(0.22, 1, 0.36, 1), background 0.6s var(--ease);
  }
  /* The surface: two wavy strips riding on top of the water, twice as wide so they can roll. */
  .wave { position: absolute; left: 0; bottom: calc(100% - 1px); width: 200%; height: 7px; fill: color-mix(in srgb, var(--c) 34%, transparent); }
  .wave.back { height: 9px; fill: color-mix(in srgb, var(--c) 18%, transparent); }
  .fixed .wave { height: 12px; }
  .fixed .wave.back { height: 16px; }
  .empty .wave { opacity: 0; }
  .slosh .wave { animation: roll 1s ease-in-out 2; }
  .slosh .wave.back { animation-direction: reverse; }
  .live .wave { animation: roll 6s linear infinite; }
  .live .wave.back { animation-duration: 9s; animation-direction: reverse; }
  @keyframes roll { to { transform: translateX(-50%); } }
  @media (prefers-reduced-motion: reduce) { .water { transition: none; } .wave { animation: none !important; } }
  :global(:root[data-motion='reduced']) .wave { animation: none !important; }
</style>
