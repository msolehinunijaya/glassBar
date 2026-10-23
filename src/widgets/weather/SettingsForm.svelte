<script lang="ts">
  import LocationPicker from '../../lib/LocationPicker.svelte';
  import type { SettingsProps } from '../types';
  import type { WeatherSettings } from './index';

  let { settings, save }: SettingsProps<WeatherSettings> = $props();
</script>

<label>Units
  <select value={settings.unit} onchange={(e) => save({ unit: e.currentTarget.value as 'c' | 'f' })}>
    <option value="c">°C</option><option value="f">°F</option>
  </select>
</label>
<label class="row"><input type="checkbox" checked={settings.colorful} onchange={(e) => save({ colorful: e.currentTarget.checked })} /> Colourful pill (by condition)</label>
<LocationPicker />
