<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { QuickSettings } from './index';

  let { settings, save }: SettingsProps<QuickSettings> = $props();
</script>

<OptionsPopover title="Quick settings sections" values={settings} save={(p) => save(p as Partial<QuickSettings>)} options={[
  { type: 'toggle', key: 'showLanguage', label: 'Keyboard language' },
  { type: 'toggle', key: 'showTray', label: 'System tray icons in the panel' },
  { type: 'toggle', key: 'trayOnDock', label: 'Show tray icons on the dock', hint: 'Next to the Wi-Fi and Bluetooth icons' },
]} />
