<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { ProgressSettings } from './index';

  let { settings, save }: SettingsProps<ProgressSettings> = $props();
</script>

<OptionsPopover title="Time progress" values={settings} save={(p) => save(p as Partial<ProgressSettings>)} options={[
  { type: 'choice', key: 'mode', label: 'On the dock', choices: [{ value: 'day', label: 'Day' }, { value: 'month', label: 'Month' }, { value: 'year', label: 'Year' }] },
]} />
