<script lang="ts">
  // The Equalizer widget is a sized box around the shared live spectrum (Spectrum.svelte),
  // which the Music widget's custom design also draws.
  import type { PillProps } from '../types';
  import type { EqSettings } from './settings';
  import Spectrum from './Spectrum.svelte';

  let { settings: s, vertical }: PillProps<EqSettings> = $props();
</script>

<div class="pill eq" class:vertical aria-label="Audio equalizer">
  <Spectrum settings={s} {vertical} len={s.length} />
</div>

<style>
  .eq { padding: 4px 8px; }
</style>
