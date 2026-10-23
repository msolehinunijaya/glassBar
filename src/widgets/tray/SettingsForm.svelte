<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { TraySettings } from './index';

  let { settings, save }: SettingsProps<TraySettings> = $props();
</script>

<OptionsPopover title="System tray" values={settings} save={(p) => save(p as Partial<TraySettings>)} options={[
  { type: 'toggle', key: 'native', label: 'Show real tray icons', hint: 'GlassBar receives icons alongside Windows. Off: the button opens Windows\' own flyout. Background apps keep running either way' },
  { type: 'toggle', key: 'showPromoted', label: 'Always-visible icons on the dock', hint: 'The ones set to show in Windows taskbar settings' },
  { type: 'toggle', key: 'showOverflow', label: 'Other icons in the overflow panel' },
  { type: 'toggle', key: 'compact', label: 'Compact (smaller, at most 4 on the dock)' },
]} />
