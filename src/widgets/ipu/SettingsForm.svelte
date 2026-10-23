<script lang="ts">
  import { ipc, type Station } from '../../lib/ipc';
  import type { SettingsProps } from '../types';
  import type { IpuSettings } from './index';
  import IpuLocation from './IpuLocation.svelte';

  let { settings, save }: SettingsProps<IpuSettings> = $props();
  let stations = $state<Station[]>([]);
  ipc.ipuGet(false).then((r) => (stations = r?.stations ?? []));
</script>

<IpuLocation {settings} {save} {stations} />
<label><input type="checkbox" checked={settings.alerts} onchange={(e) => save({ alerts: e.currentTarget.checked })} /> Alert when air becomes unhealthy (above 100, and above 200)</label>
