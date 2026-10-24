<script lang="ts">
  // Standalone Start & Search widget (hidden while the App Launcher shows them inline).
  import type { PillProps } from '../types';
  import StartButtons from './StartButtons.svelte';

  let {}: PillProps = $props();
</script>

<div class="pill start"><StartButtons /></div>

<style>
  .start { gap: 2px; padding: 0 3px; }
</style>
