<script lang="ts">
  import type { SettingsProps } from '../types';
  import type { ClockSettings } from './index';
  import FormatOptions from './FormatOptions.svelte';
  import { zonesOf } from './zones';

  let { settings, save }: SettingsProps<ClockSettings> = $props();
  const zones = $derived(zonesOf(settings));
</script>

<FormatOptions {settings} {save} />
<p class="dim" style="font-size:12px;margin:0">
  World clocks: {zones.length ? zones.map((z) => z.label).join(', ') : 'none yet'}. Add, remove and reorder them in the clock's panel (click the clock, then +).
</p>
