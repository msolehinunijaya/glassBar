<script lang="ts">
  import { ipc } from '../../lib/ipc';
  import LocationPicker from '../../lib/LocationPicker.svelte';
  import type { SettingsProps } from '../types';
  import type { SolatSettings } from './index';

  let { settings, save }: SettingsProps<SolatSettings> = $props();
  let zones = $state<[string, string, string][]>([]);
  ipc.solatZones().then((z) => (zones = z));
</script>

<label>Zone
  <select value={settings.zone} onchange={(e) => save({ zone: e.currentTarget.value })}>
    <option value="auto">Auto (from my location)</option>
    {#each zones as [code, state, district] (code)}<option value={code}>{code} – {district} ({state})</option>{/each}
  </select>
</label>
{#if settings.zone === 'auto'}<LocationPicker />{/if}
<label><input type="checkbox" checked={settings.showImsak} onchange={(e) => save({ showImsak: e.currentTarget.checked })} /> Show Imsak</label>
<label><input type="checkbox" checked={settings.alerts} onchange={(e) => save({ alerts: e.currentTarget.checked })} /> Alert at each prayer time</label>
<label>Azan sound file (MP3/WAV, optional)
  <input style="flex:1" placeholder="C:\Users\me\Music\azan.mp3" value={settings.azanFile}
         onchange={(e) => save({ azanFile: e.currentTarget.value.trim().replace(/^"|"$/g, '') })} />
  <button class="btn" disabled={!settings.azanFile} onclick={() => ipc.solatTestSound(settings.azanFile)}>Test</button>
</label>
