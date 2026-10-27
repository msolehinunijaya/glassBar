<script lang="ts">
  // Standalone Start & Search widget (hidden while the App Launcher shows them inline).
  import type { PillProps } from '../types';
  import StartButtons from './StartButtons.svelte';
  import { profile } from '../../lib/config.svelte';
  import type { AppsSettings } from '../apps/index';

  let {}: PillProps = $props();
  // The logo colour is chosen in the App Launcher's Start & Search options: the same here.
  const apps = $derived(profile().widgets.find((w) => w.id === 'apps')?.settings as Partial<AppsSettings> | undefined);
</script>

<div class="pill start"><StartButtons color={apps?.startColor} custom={apps?.startCustom} /></div>

<style>
  .start { gap: 2px; padding: 0 3px; }
</style>
