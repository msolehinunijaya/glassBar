<script lang="ts">
  import type { SettingsProps } from '../types';
  import type { ClockSettings } from './index';

  let { settings, save }: SettingsProps<ClockSettings> = $props();
  const zones = Intl.supportedValuesOf('timeZone');
</script>

<label><input type="checkbox" checked={settings.hour12} onchange={(e) => save({ hour12: e.currentTarget.checked })} /> 12-hour clock</label>
<label><input type="checkbox" checked={settings.showSeconds} onchange={(e) => save({ showSeconds: e.currentTarget.checked })} /> Show seconds</label>
<label>
  Second time zone
  <input list="tz-list" placeholder="e.g. Asia/Tokyo" value={settings.secondTz}
         onchange={(e) => save({ secondTz: e.currentTarget.value.trim() })} />
  <datalist id="tz-list">{#each zones as z (z)}<option value={z}></option>{/each}</datalist>
</label>
