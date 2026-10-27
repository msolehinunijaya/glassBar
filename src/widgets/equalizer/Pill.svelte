<script lang="ts">
  // The Equalizer widget is a sized box around the shared live spectrum (Spectrum.svelte),
  // which the Music widget's custom design also draws.
  import type { PillProps } from '../types';
  import { SHRINK_AFTER_MS, shrunkAt, type EqSettings } from './settings';
  import Spectrum from './Spectrum.svelte';

  let { settings: s, vertical }: PillProps<EqSettings> = $props();

  // Nothing playing: after SHRINK_AFTER_MS the pill slides down to a small square (or away,
  // with the idle look "Hide") to give the dock its room back; sound grows it back at once.
  let quietSince = $state<number | null>(null);
  let now = $state(0);
  let timer: ReturnType<typeof setTimeout>;
  function onquiet(q: boolean) {
    clearTimeout(timer);
    quietSince = q ? performance.now() : null;
    now = performance.now();
    if (q) timer = setTimeout(() => (now = performance.now()), SHRINK_AFTER_MS);
  }
  $effect(() => () => clearTimeout(timer));
  const shrunk = $derived(s.shrinkIdle !== false && shrunkAt(quietSince, now));
  const gone = $derived(shrunk && s.idle === 'hide');
  // The radial ring is always a square as big as the dock is thick, and so is a shrunk pill:
  // measured in px (not CSS auto) so the change in length can slide.
  let box = $state<DOMRectReadOnly>();
  const thick = $derived(Math.round((vertical ? box?.width : box?.height) ?? 0));
  // Hidden is 1 px, never 0: the frame loop skips a 0 px canvas before it checks for sound,
  // so a hidden equalizer would never notice the music again.
  const len = $derived(gone ? 1 : s.style === 'radial' || shrunk ? thick : s.length);
</script>

<!-- The vertical layout comes from `.dock.vertical .pill` in glass.css; the canvas gets it
     from the `vertical` prop. There is nothing left for a class on this div to do. -->
<div class="pill eq" class:shrunk class:gone bind:contentRect={box} aria-label={shrunk ? 'Audio equalizer (nothing playing)' : 'Audio equalizer'}>
  <Spectrum settings={s} {vertical} {len} {onquiet} />
</div>

<style>
  .eq { padding: 4px 8px; transition: padding 0.26s var(--spring, ease), opacity 0.2s var(--ease); }
  .eq.shrunk { padding: 4px; }
  .eq.gone { padding: 0; opacity: 0; }
  @media (prefers-reduced-motion: reduce) { .eq { transition: none; } }
</style>
