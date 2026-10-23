<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { SysmonSettings } from './index';

  let { settings, save }: SettingsProps<SysmonSettings> = $props();
</script>

<OptionsPopover title="Show on the bar" values={settings} save={(p) => save(p as Partial<SysmonSettings>)} options={[
  { type: 'checks', label: 'Items', min: 1, items: [{ key: 'cpu', label: 'CPU' }, { key: 'ram', label: 'RAM' }, { key: 'network', label: 'Network' }, { key: 'disk', label: 'Disk' }] },
  { type: 'choice', key: 'diskHigh', label: 'Disk orange from', choices: [{ value: '70', label: '70%' }, { value: '80', label: '80%' }, { value: '85', label: '85%' }] },
  { type: 'choice', key: 'diskCritical', label: 'Disk red from', choices: [{ value: '90', label: '90%' }, { value: '95', label: '95%' }] },
  { type: 'toggle', key: 'temp', label: 'CPU temperature after CPU %', hint: 'From Windows thermal zones; N/A if this PC exposes none' },
  { type: 'choice', key: 'tempUnit', label: 'Unit', choices: [{ value: 'c', label: '°C' }, { value: 'f', label: '°F' }] },
  { type: 'choice', key: 'tempWarn', label: 'Warn at', choices: [{ value: '70', label: '70°C' }, { value: '80', label: '80°C' }, { value: '90', label: '90°C' }] },
  { type: 'toggle', key: 'gpu', label: 'GPU graph in the panel' },
  { type: 'toggle', key: 'recycle', label: 'Recycle Bin size on the bar', hint: 'Checked once a minute' },
]} />
