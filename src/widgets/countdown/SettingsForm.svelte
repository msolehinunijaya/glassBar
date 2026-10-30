<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { CountdownSettings } from './index';

  let { settings, save }: SettingsProps<CountdownSettings> = $props();
</script>

<OptionsPopover title="Countdowns" values={settings} save={(p) => save(p as Partial<CountdownSettings>)} options={[
  { type: 'toggle', key: 'notify', label: 'Notify on the day', hint: 'At the set time, or 9:00. Add a day-before reminder per countdown.' },
]} />
