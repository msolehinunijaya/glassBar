<script lang="ts">
  import type { SettingsProps } from '../types';
  import type { DownloadsSettings } from './index';
  import Options from './Options.svelte';

  let { settings, save }: SettingsProps<DownloadsSettings> = $props();
</script>

<Options {settings} {save} />
<p class="dim" style="font-size:12px;margin:6px 0 0">Browsers that save somewhere else only show here if you pick that folder.</p>
