<script lang="ts">
  import type { SettingsProps } from '../types';
  import type { HelloSettings } from './index';

  // save() merges the patch into this widget instance's settings and persists it.
  let { settings, save }: SettingsProps<HelloSettings> = $props();
</script>

<label>Name <input value={settings.name} onchange={(e) => save({ name: e.currentTarget.value })} /></label>
