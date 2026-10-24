<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { TimerSettings } from './index';

  let { settings, save }: SettingsProps<TimerSettings> = $props();
</script>

<OptionsPopover title="On the dock" values={{ ...settings, drain: settings.drain ? 'drain' : 'fill' }}
  save={(p) => save((p.drain !== undefined ? { drain: p.drain === 'drain' } : p) as Partial<TimerSettings>)} options={[
  { type: 'toggle', key: 'progress', label: 'Progress background', hint: 'The pill fills up as the timer runs' },
  { type: 'choice', key: 'drain', label: 'Direction', choices: [{ value: 'fill', label: 'Fill up', hint: 'Time elapsed' }, { value: 'drain', label: 'Drain', hint: 'Time remaining' }] },
  { type: 'toggle', key: 'warn', label: 'Orange in the final minute' },
  { type: 'toggle', key: 'shimmer', label: 'Shimmer while the stopwatch runs' },
]} />
