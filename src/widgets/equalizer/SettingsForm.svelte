<script lang="ts">
  // Every Equalizer setting, grouped, with presets on top. Changes save at once and the bar
  // redraws live. Options that don't apply to the chosen look are hidden.
  import { X } from '@lucide/svelte';
  import OptionsPopover, { type Option } from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import { applyPreset, pickLook, PRESETS, type EqSettings, type Preset } from './settings';

  let { settings: s, save }: SettingsProps<EqSettings> = $props();
  const put = (p: Record<string, unknown>) => save(p as Partial<EqSettings>);

  let name = $state('');
  const saveCurrent = () => {
    const n = name.trim() || `My preset ${s.myPresets.length + 1}`;
    save({ myPresets: [...s.myPresets.filter((p) => p.name !== n), { name: n, values: pickLook(s) }] });
    name = '';
  };
  const remove = (p: Preset) => save({ myPresets: s.myPresets.filter((q) => q !== p) });

  const opt = (on: boolean, ...o: Option[]) => (on ? o : []);
  const reflects = $derived(!['mirror', 'radial', 'particles'].includes(s.style));
  const barLike = $derived(['bars', 'mirror', 'glow', 'led'].includes(s.style));

  const sections = $derived<[string, Option[]][]>([
    ['Style', [
      { type: 'choice', key: 'style', label: 'Look', menu: true, choices: [
        { value: 'bars', label: 'Bars' },
        { value: 'mirror', label: 'Mirror', hint: 'Grows up and down from the middle' },
        { value: 'glow', label: 'Glow spikes', hint: 'Thin neon lines with a soft haze' },
        { value: 'led', label: 'LED blocks', hint: 'Stacked segments, like an old stereo' },
        { value: 'particles', label: 'Particle wave', hint: 'Stacked dotted waves' },
        { value: 'line', label: 'Line / area', hint: 'A smooth curve through the bar tops' },
        { value: 'radial', label: 'Radial ring', hint: 'Bars around a circle; best with Length ≈ the bar’s height' }] },
      ...opt(s.style === 'led', { type: 'range', key: 'ledSize', label: 'Segment', min: 1, max: 8, unit: 'px' }),
      ...opt(s.style === 'particles', { type: 'range', key: 'layers', label: 'Layers', min: 1, max: 8 }),
      ...opt(s.style === 'line',
        { type: 'range', key: 'lineWidth', label: 'Line width', min: 1, max: 5, step: 0.5, unit: 'px' },
        { type: 'toggle', key: 'area', label: 'Fill under the line' }),
      { type: 'toggle', key: 'flip', label: 'Treble on the left' },
    ]],
    ['Size', [
      { type: 'range', key: 'length', label: 'Length', min: 30, max: 400, step: 2, unit: 'px' },
      { type: 'range', key: 'fill', label: 'Height', min: 30, max: 100, step: 5, unit: '%' },
      { type: 'range', key: 'bands', label: 'Bars', min: 8, max: 128 },
      { type: 'range', key: 'gap', label: 'Gap', min: 0, max: 8, step: 0.5, unit: 'px' },
      ...opt(barLike || s.style === 'radial', { type: 'range', key: 'radius', label: 'Corners', min: 0, max: 6, step: 0.5, unit: 'px' }),
    ]],
    ['Colour', [
      { type: 'choice', key: 'colorMode', label: 'Colour', menu: true, choices: [
        { value: 'accent', label: 'Accent colour' }, { value: 'solid', label: 'Solid' },
        { value: 'gradient', label: 'Gradient' }, { value: 'rainbow', label: 'Rainbow' },
        { value: 'album', label: 'Album art', hint: 'Two colours from the playing song’s cover' }] },
      ...opt(s.colorMode === 'solid' || s.colorMode === 'gradient', { type: 'color', key: 'color1', label: s.colorMode === 'solid' ? 'Colour' : 'From' }),
      ...opt(s.colorMode === 'gradient', { type: 'color', key: 'color2', label: 'To' }),
      ...opt(s.colorMode !== 'solid', { type: 'choice', key: 'direction', label: 'Direction', choices: [
        { value: 'across', label: 'Across bars' }, { value: 'up', label: 'Bottom to top' }] }),
      ...opt(s.colorMode === 'rainbow', { type: 'range', key: 'hueSpeed', label: 'Colour cycle', min: 0, max: 120, step: 5, unit: '°/s' }),
    ]],
    ['Glow', [
      { type: 'toggle', key: 'glow', label: 'Glow' },
      ...opt(s.glow,
        { type: 'range', key: 'glowSize', label: 'Blur', min: 1, max: 20, unit: 'px' },
        { type: 'range', key: 'glowStrength', label: 'Intensity', min: 10, max: 100, step: 5, unit: '%' },
        { type: 'choice', key: 'glowColor', label: 'Glow colour', choices: [{ value: 'match', label: 'Match bars' }, { value: 'custom', label: 'Custom' }] }),
      ...opt(s.glow && s.glowColor === 'custom', { type: 'color', key: 'glowCustom', label: 'Custom glow' }),
    ]],
    ...(reflects ? [['Reflection', [
      { type: 'toggle', key: 'reflection', label: 'Reflection', hint: 'A faded mirror image under the bars' },
      ...opt(s.reflection,
        { type: 'range', key: 'reflStrength', label: 'Strength', min: 10, max: 80, step: 5, unit: '%' },
        { type: 'range', key: 'reflSize', label: 'Size', min: 15, max: 50, step: 5, unit: '%' }),
    ]] as [string, Option[]]] : []),
    ['Opacity', [
      { type: 'range', key: 'barOpacity', label: 'Bars', min: 10, max: 100, step: 5, unit: '%' },
      { type: 'range', key: 'bgOpacity', label: 'Background', min: 0, max: 100, step: 5, unit: '%' },
      { type: 'range', key: 'idleOpacity', label: 'When idle', min: 0, max: 100, step: 5, unit: '%' },
    ]],
    ['Motion', [
      { type: 'range', key: 'gain', label: 'Sensitivity', min: 50, max: 300, step: 10, unit: '%' },
      { type: 'range', key: 'fall', label: 'Fall speed', min: 1, max: 10 },
      ...opt(s.style !== 'particles', { type: 'toggle', key: 'peaks', label: 'Peak caps', hint: 'A small marker hangs at each bar’s high point' }),
      ...opt(s.style !== 'particles' && s.peaks, { type: 'range', key: 'peakHold', label: 'Cap hold', min: 0, max: 2000, step: 100, unit: 'ms' }),
    ]],
    ['Frequency', [
      { type: 'choice', key: 'lowHz', label: 'From', choices: [{ value: '20', label: '20 Hz' }, { value: '30', label: '30 Hz' }, { value: '60', label: '60 Hz' }, { value: '120', label: '120 Hz' }] },
      { type: 'choice', key: 'highHz', label: 'To', choices: [{ value: '4000', label: '4 kHz' }, { value: '8000', label: '8 kHz' }, { value: '12000', label: '12 kHz' }, { value: '16000', label: '16 kHz' }, { value: '20000', label: '20 kHz' }] },
      { type: 'range', key: 'tilt', label: 'Bass ⟷ Treble', min: -100, max: 100, step: 5 },
      { type: 'choice', key: 'spacing', label: 'Spacing', choices: [{ value: 'log', label: 'Musical', hint: 'Log scale, like your ears' }, { value: 'linear', label: 'Linear' }] },
    ]],
    ['Idle & power', [
      { type: 'choice', key: 'idle', label: 'Nothing playing', choices: [{ value: 'dots', label: 'Dots' }, { value: 'line', label: 'Line' }, { value: 'hide', label: 'Hide' }] },
      { type: 'choice', key: 'fps', label: 'Frame rate', choices: [{ value: '30', label: '30 fps' }, { value: '60', label: '60 fps' }] },
      { type: 'toggle', key: 'batterySaver', label: 'Pause on battery', hint: 'Stops listening while unplugged' },
    ]],
  ]);
</script>

<section class="presets">
  <h4>Presets</h4>
  <div class="chips">
    {#each PRESETS as p (p.name)}
      <button class="chip" onclick={() => save(applyPreset(p))}>{p.name}</button>
    {/each}
    {#each s.myPresets as p (p.name)}
      <span class="chip mine">
        <button onclick={() => save(applyPreset(p))}>{p.name}</button>
        <button class="del" aria-label="Delete preset {p.name}" title="Delete" onclick={() => remove(p)}><X size={12} /></button>
      </span>
    {/each}
  </div>
  <form class="savep" onsubmit={(e) => { e.preventDefault(); saveCurrent(); }}>
    <input bind:value={name} placeholder="Name" maxlength="24" aria-label="Preset name" />
    <button class="chip" type="submit">Save current</button>
  </form>
</section>

{#each sections as [title, options] (title)}
  <OptionsPopover {title} {options} values={s} save={put} />
{/each}

<style>
  .presets { display: flex; flex-direction: column; gap: var(--s-2); padding: 0 var(--s-1) var(--s-3); margin-bottom: var(--s-3); border-bottom: 1px solid var(--border); }
  h4 { margin: 0; font-size: 12px; font-weight: 600; color: var(--fg-dim); }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; }
  .chip { padding: 5px 10px; border-radius: var(--r-inner); background: var(--bg-2); border: 1px solid var(--border); font-size: 12px; white-space: nowrap; }
  .chip:hover, .mine button:hover { background: var(--pill-hover); }
  .mine { display: inline-flex; align-items: center; padding: 0; overflow: hidden; }
  .mine button { padding: 5px 4px 5px 10px; font-size: 12px; }
  .mine .del { padding: 5px 6px; display: flex; color: var(--fg-dim); }
  .chip:focus-visible, .mine button:focus-visible, input:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
  .savep { display: flex; gap: 6px; }
  .savep input { flex: 1; min-width: 0; height: 30px; padding: 0 10px; border-radius: var(--r-inner); border: 1px solid var(--border); background: var(--bg-2); color: inherit; font: inherit; font-size: 12px; }
</style>
