<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { ClockSettings } from './index';
  import { zonesOf } from './zones';

  let { settings, save }: SettingsProps<ClockSettings> = $props();
  const zones = $derived(zonesOf(settings));
</script>

<OptionsPopover title="Clock" values={{ ...settings, format: settings.hour12 ? '12' : '24' }}
  save={(p) => save(('format' in p ? { hour12: p.format === '12' } : p) as Partial<ClockSettings>)} options={[
    { type: 'choice', key: 'appearance', label: 'Appearance', choices: [
      { value: 'standard', label: 'Standard' }, { value: 'prominent', label: 'Prominent', hint: 'Bigger, bolder time on an accent panel' },
      { value: 'popup', label: 'Popup', hint: 'Hover the clock to see a large clock and world clocks' }] },
    { type: 'choice', key: 'format', label: 'Format', choices: [{ value: '24', label: '24-hour' }, { value: '12', label: '12-hour' }] },
    { type: 'toggle', key: 'showSeconds', label: 'Show seconds' },
]} />
<p class="dim" style="font-size:12px;margin:0">
  World clocks: {zones.length ? zones.map((z) => z.label).join(', ') : 'none yet'}. Add, remove and reorder them in the clock's panel (click the clock, then +).
</p>
