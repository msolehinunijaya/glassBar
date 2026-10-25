<script lang="ts">
  // Every Equalizer setting, grouped. Changes save at once and the bar redraws live.
  import OptionsPopover, { type Option } from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { EqSettings } from './index';

  let { settings: s, save }: SettingsProps<EqSettings> = $props();
  const put = (p: Record<string, unknown>) => save(p as Partial<EqSettings>);

  const sections = $derived<[string, Option[]][]>([
    ['Style', [
      { type: 'choice', key: 'style', label: 'Look', choices: [
        { value: 'bars', label: 'Bars' }, { value: 'mirror', label: 'Mirror', hint: 'Grows up and down from the middle' },
        { value: 'glow', label: 'Glow spikes', hint: 'Thin neon lines with a soft haze' }] },
      { type: 'toggle', key: 'flip', label: 'Treble on the left' },
    ]],
    ['Size', [
      { type: 'range', key: 'length', label: 'Length', min: 40, max: 400, step: 4, unit: 'px' },
      { type: 'range', key: 'fill', label: 'Height', min: 30, max: 100, step: 5, unit: '%' },
      { type: 'range', key: 'bands', label: 'Bars', min: 8, max: 128 },
      { type: 'range', key: 'gap', label: 'Gap', min: 0, max: 8, step: 0.5, unit: 'px' },
      { type: 'range', key: 'radius', label: 'Corners', min: 0, max: 6, step: 0.5, unit: 'px' },
    ]],
    ['Colour', [
      { type: 'choice', key: 'colorMode', label: 'Colour', choices: [
        { value: 'accent', label: 'Accent' }, { value: 'solid', label: 'Solid' },
        { value: 'gradient', label: 'Gradient' }, { value: 'rainbow', label: 'Rainbow' }] },
      ...(s.colorMode === 'solid' || s.colorMode === 'gradient' ? [{ type: 'color', key: 'color1', label: s.colorMode === 'solid' ? 'Colour' : 'From' } as Option] : []),
      ...(s.colorMode === 'gradient' ? [{ type: 'color', key: 'color2', label: 'To' } as Option] : []),
      ...(s.colorMode !== 'solid' ? [{ type: 'choice', key: 'direction', label: 'Direction', choices: [
        { value: 'across', label: 'Across bars' }, { value: 'up', label: 'Bottom to top' }] } as Option] : []),
      ...(s.colorMode === 'rainbow' ? [{ type: 'range', key: 'hueSpeed', label: 'Colour cycle', min: 0, max: 120, step: 5, unit: '°/s' } as Option] : []),
    ]],
    ['Glow', [
      { type: 'toggle', key: 'glow', label: 'Glow' },
      ...(s.glow ? [
        { type: 'range', key: 'glowSize', label: 'Blur', min: 1, max: 20, unit: 'px' },
        { type: 'range', key: 'glowStrength', label: 'Intensity', min: 10, max: 100, step: 5, unit: '%' },
        { type: 'choice', key: 'glowColor', label: 'Glow colour', choices: [{ value: 'match', label: 'Match bars' }, { value: 'custom', label: 'Custom' }] },
        ...(s.glowColor === 'custom' ? [{ type: 'color', key: 'glowCustom', label: 'Custom glow' }] : []),
      ] as Option[] : []),
    ]],
    ['Opacity', [
      { type: 'range', key: 'barOpacity', label: 'Bars', min: 10, max: 100, step: 5, unit: '%' },
      { type: 'range', key: 'bgOpacity', label: 'Background', min: 0, max: 100, step: 5, unit: '%' },
      { type: 'range', key: 'idleOpacity', label: 'When idle', min: 0, max: 100, step: 5, unit: '%' },
    ]],
    ['Motion', [
      { type: 'range', key: 'gain', label: 'Sensitivity', min: 50, max: 300, step: 10, unit: '%' },
      { type: 'range', key: 'fall', label: 'Fall speed', min: 1, max: 10 },
      { type: 'toggle', key: 'peaks', label: 'Peak caps', hint: 'A small marker hangs at each bar’s high point' },
      ...(s.peaks ? [{ type: 'range', key: 'peakHold', label: 'Cap hold', min: 0, max: 2000, step: 100, unit: 'ms' } as Option] : []),
    ]],
    ['Frequency', [
      { type: 'choice', key: 'lowHz', label: 'From', choices: [{ value: '20', label: '20 Hz' }, { value: '30', label: '30 Hz' }, { value: '60', label: '60 Hz' }, { value: '120', label: '120 Hz' }] },
      { type: 'choice', key: 'highHz', label: 'To', choices: [{ value: '4000', label: '4 kHz' }, { value: '8000', label: '8 kHz' }, { value: '12000', label: '12 kHz' }, { value: '16000', label: '16 kHz' }, { value: '20000', label: '20 kHz' }] },
      { type: 'range', key: 'tilt', label: 'Bass ⟷ Treble', min: -100, max: 100, step: 5 },
      { type: 'choice', key: 'spacing', label: 'Spacing', choices: [{ value: 'log', label: 'Musical', hint: 'Log scale, like your ears' }, { value: 'linear', label: 'Linear' }] },
    ]],
    ['Idle & performance', [
      { type: 'choice', key: 'idle', label: 'Nothing playing', choices: [{ value: 'dots', label: 'Dots' }, { value: 'line', label: 'Line' }, { value: 'hide', label: 'Hide' }] },
      { type: 'choice', key: 'fps', label: 'Frame rate', choices: [{ value: '30', label: '30 fps' }, { value: '60', label: '60 fps' }] },
    ]],
  ]);
</script>

{#each sections as [title, options] (title)}
  <OptionsPopover {title} {options} values={s} save={put} />
{/each}
