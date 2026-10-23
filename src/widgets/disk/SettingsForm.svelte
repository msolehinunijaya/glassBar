<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { DiskSettings } from './index';

  let { settings, save }: SettingsProps<DiskSettings> = $props();
</script>

<OptionsPopover title="Disk storage" values={settings} save={(p) => save(p as Partial<DiskSettings>)} options={[
  { type: 'choice', key: 'high', label: 'Orange from', choices: [{ value: '70', label: '70%' }, { value: '80', label: '80%' }, { value: '85', label: '85%' }] },
  { type: 'choice', key: 'critical', label: 'Red from', choices: [{ value: '90', label: '90%' }, { value: '95', label: '95%' }] },
  { type: 'toggle', key: 'showFree', label: 'Show free space instead of % (medium size)' },
]} />
<p class="dim" style="font-size:12px">Pick which drives appear on the bar in the widget's panel.</p>
