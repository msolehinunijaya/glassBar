<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { MirrorSettings } from './mirror';

  let { settings, save }: SettingsProps<MirrorSettings> = $props();
</script>

<OptionsPopover title="Mirror" values={settings} save={(p) => save(p as Partial<MirrorSettings>)} options={[
  { type: 'choice', key: 'shape', label: 'Shape', choices: [{ value: 'circle', label: 'Circle' }, { value: 'square', label: 'Square' }, { value: 'rect', label: 'Rectangle' }] },
  { type: 'toggle', key: 'flip', label: 'Flip like a mirror' },
  { type: 'range', key: 'brightness', label: 'Brightness', min: 50, max: 200, unit: '%' },
  { type: 'range', key: 'contrast', label: 'Contrast', min: 50, max: 200, unit: '%' },
  { type: 'range', key: 'ring', label: 'Ring light', min: 0, max: 100, unit: '%' },
  { type: 'range', key: 'zoom', label: 'Zoom', min: 1, max: 3, step: 0.1, unit: '×' },
  { type: 'toggle', key: 'grid', label: 'Grid' },
  { type: 'toggle', key: 'countdown', label: '3-second timer before a photo' },
]} />
<p class="dim" style="font-size:11px;margin:8px 4px 0">Photos are saved in Pictures › GlassBar Mirror, videos in Videos › GlassBar Mirror.</p>
