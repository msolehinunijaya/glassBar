<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { NotifSettings } from './index';

  let { settings, save }: SettingsProps<NotifSettings> = $props();
</script>

<OptionsPopover title="Notifications" values={settings} save={(p) => save(p as Partial<NotifSettings>)} options={[
  { type: 'toggle', key: 'badge', label: 'Unread badge on the bar' },
  { type: 'toggle', key: 'windowsTab', label: 'Show the Windows tab', hint: 'Opens Windows Notification Center' },
  { type: 'toggle', key: 'rememberTab', label: 'Remember the last tab' },
]} />
