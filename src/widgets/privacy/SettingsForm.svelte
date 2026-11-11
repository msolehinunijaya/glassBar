<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { PrivacySettings } from './settings';

  let { settings, save }: SettingsProps<PrivacySettings> = $props();
  const put = (p: Record<string, unknown>) => save(p as Partial<PrivacySettings>);
</script>

<OptionsPopover title="Monitor" values={settings} save={put} options={[
  { type: 'checks', label: 'Show', min: 1, items: [
    { key: 'showCamera', label: 'Camera' }, { key: 'showMic', label: 'Microphone' }, { key: 'showLocation', label: 'Location' },
    { key: 'showRecording', label: 'Screen recording' }, { key: 'showSharing', label: 'Screen sharing' }] },
]} />

<OptionsPopover title="Look" values={settings} save={put} options={[
  { type: 'toggle', key: 'glow', label: 'Glow while in use', hint: 'A soft glow around the dock icon' },
  { type: 'choice', key: 'tone', label: 'Colour while in use', choices: [
    { value: 'status', label: 'Per resource', hint: 'Camera green, microphone orange, location blue, screen red or purple' },
    { value: 'accent', label: 'Accent', hint: 'The accent colour from Settings › Appearance' }] },
  { type: 'toggle', key: 'details', label: 'Open card details by default' },
  { type: 'toggle', key: 'history', label: 'Recent activity', hint: 'Windows\' own record of the last 7 days. GlassBar stores nothing.' },
]} />

<OptionsPopover title="Notify me" values={settings} save={put} options={[
  { type: 'toggle', key: 'notifyCamera', label: 'When an app starts using the camera' },
  { type: 'toggle', key: 'notifyMic', label: 'When an app starts using the microphone' },
  { type: 'toggle', key: 'notifyLocation', label: 'When an app starts using location', hint: 'Only where Windows lets apps see it (not on recent Windows 11)' },
  { type: 'toggle', key: 'notifyScreen', label: 'When an app starts capturing the screen', hint: 'Only where Windows lets apps see it (not on recent Windows 11)' },
  { type: 'toggle', key: 'notifyProblems', label: 'When something can\'t be checked or is turned off', hint: 'A device becomes unavailable, location or its service is turned off' },
]} />
