<script lang="ts">
  // The Equalizer widget is a sized box around the shared live spectrum (Spectrum.svelte),
  // which the Music widget's custom design also draws.
  import type { PillProps } from '../types';
  import type { EqSettings } from './settings';
  import Spectrum from './Spectrum.svelte';

  let { settings: s, vertical }: PillProps<EqSettings> = $props();
</script>

<!-- The vertical layout comes from `.dock.vertical .pill` in glass.css; the canvas gets it
     from the `vertical` prop. There is nothing left for a class on this div to do. -->
<div class="pill eq" aria-label="Audio equalizer">
  <Spectrum settings={s} {vertical} len={s.length} />
</div>

<style>
  .eq { padding: 4px 8px; }
</style>
