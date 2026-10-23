<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { SysmonSettings } from './index';

  let { settings, save }: SettingsProps<SysmonSettings> = $props();
</script>

<OptionsPopover title="Show on the bar" values={settings} save={(p) => save(p as Partial<SysmonSettings>)} options={[
  { type: 'checks', label: 'Items', min: 1, items: [{ key: 'cpu', label: 'CPU' }, { key: 'ram', label: 'RAM' }, { key: 'network', label: 'Network' }] },
  { type: 'toggle', key: 'gpu', label: 'GPU graph in the panel' },
]} />
