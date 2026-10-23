<script lang="ts">
  import { ipc } from '../../lib/ipc';
  import LocationPicker from '../../lib/LocationPicker.svelte';
  import type { SettingsProps } from '../types';
  import type { IpuSettings } from './index';

  let { settings, save }: SettingsProps<IpuSettings> = $props();
  let stations = $state<[string, string][]>([]);
  ipc.ipuGet(false).then((r) => (stations = [...(r?.stations ?? [])].sort((a, b) => a[1].localeCompare(b[1]))));
</script>

<label>Station
  <select value={settings.station} onchange={(e) => save({ station: e.currentTarget.value })}>
    <option value="auto">Nearest to my location</option>
    {#each stations as [id, name] (id)}<option value={id}>{name}</option>{/each}
  </select>
</label>
{#if settings.station === 'auto'}<LocationPicker />{/if}
<label><input type="checkbox" checked={settings.alerts} onchange={(e) => save({ alerts: e.currentTarget.checked })} /> Alert when air becomes unhealthy (IPU above 100, and above 200)</label>
