<script lang="ts">
  // Weather on the clock (was the Weather widget): on/off, units, coloured background.
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { ClockSettings } from './index';

  let { settings, save }: { settings: ClockSettings; save: (p: Partial<ClockSettings>) => void } = $props();
</script>

<OptionsPopover title="Weather" values={settings} save={(p) => save(p as Partial<ClockSettings>)} options={[
  { type: 'toggle', key: 'showWeather', label: 'Show weather on the clock', hint: 'Icon and temperature on the left; click it for the forecast' },
  ...(settings.showWeather ? [
    { type: 'choice' as const, key: 'weatherUnit', label: 'Units', choices: [{ value: 'c', label: '°C' }, { value: 'f', label: '°F' }] },
    { type: 'toggle' as const, key: 'weatherColorful', label: 'Coloured by the weather', hint: 'Sunny, cloudy, rain… each has its own colour' },
  ] : []),
]} />
