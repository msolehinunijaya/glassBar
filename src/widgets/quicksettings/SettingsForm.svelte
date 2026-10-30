<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import { TILES, type QuickSettings, type TileKey } from './index';

  let { settings, save }: SettingsProps<QuickSettings> = $props();
  // Edit tiles: a box per tile, ticked = shown. Stored as the hidden ones, so a tile added
  // later shows up by default.
  const shown = $derived(Object.fromEntries(TILES.map(([k]) => [k, !(settings.hiddenTiles ?? []).includes(k)])));
  const saveTiles = (p: Record<string, unknown>) => {
    const next = { ...shown, ...p };
    save({ hiddenTiles: TILES.map(([k]) => k).filter((k) => !next[k]) as TileKey[] });
  };
</script>

<OptionsPopover title="Edit tiles" values={shown} save={saveTiles} options={[
  { type: 'checks', label: 'Show', min: 1, items: TILES.map(([key, label]) => ({ key, label })) },
]} />

<OptionsPopover title="Quick settings sections" values={settings} save={(p) => save(p as Partial<QuickSettings>)} options={[
  { type: 'toggle', key: 'showLanguage', label: 'Keyboard language' },
  { type: 'toggle', key: 'langOnDock', label: 'Show keyboard language on the dock', hint: 'EN, MS… next to the Wi-Fi icon' },
  { type: 'toggle', key: 'soundOnDock', label: 'Volume on the dock', hint: 'Click the speaker to mute, hover to adjust, click the % for the sound panel' },
  { type: 'toggle', key: 'showTray', label: 'System tray icons in the panel' },
  { type: 'choice', key: 'micOnDock', label: 'Microphone on the dock', choices: [
    { value: 'auto', label: 'Automatic', hint: 'Only when muted or in use' }, { value: 'always', label: 'Always' }, { value: 'never', label: 'Never' }] },
  { type: 'toggle', key: 'btBattery', label: 'Bluetooth device battery', hint: 'Headphones, phones and other devices that report it' },
  { type: 'toggle', key: 'btLowAlert', label: 'Notify when a device battery is low', hint: 'Once at 15 %' },
  { type: 'toggle', key: 'trayOnDock', label: 'Show tray icons on the dock', hint: 'Next to the Wi-Fi and Bluetooth icons' },
]} />
