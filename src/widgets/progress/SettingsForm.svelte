<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { ProgressSettings } from './index';

  let { settings, save }: SettingsProps<ProgressSettings> = $props();
</script>

<OptionsPopover title="Time progress" values={settings} save={(p) => save(p as Partial<ProgressSettings>)} options={[
  { type: 'choice', key: 'mode', label: 'Mode', choices: [{ value: 'day', label: 'Day' }, { value: 'month', label: 'Month' }, { value: 'year', label: 'Year' }] },
  { type: 'toggle', key: 'showTimes', label: 'Show elapsed and remaining time' },
]} />
