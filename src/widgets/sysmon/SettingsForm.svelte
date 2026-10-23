<script lang="ts">
  import type { SettingsProps } from '../types';
  import type { SysmonSettings } from './index';

  let { settings, save }: SettingsProps<SysmonSettings> = $props();
</script>

<label><input type="checkbox" checked={settings.gpu} onchange={(e) => save({ gpu: e.currentTarget.checked })} /> Show GPU usage</label>
<label><input type="checkbox" checked={settings.network} onchange={(e) => save({ network: e.currentTarget.checked })} /> Show network speed</label>
